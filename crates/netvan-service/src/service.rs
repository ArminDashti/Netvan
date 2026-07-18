use anyhow::{bail, Result};
use std::ffi::OsString;
use std::process::Command;
use tracing::info;

pub const SERVICE_NAME: &str = "NetvanService";
pub const SERVICE_DISPLAY: &str = "Netvan Network Monitor";

pub fn install() -> Result<()> {
    let exe = std::env::current_exe()?;
    let bin = exe.display().to_string();
    // Use sc.exe for simple service registration pointing at this binary with `run`
    let status = Command::new("sc")
        .args([
            "create",
            SERVICE_NAME,
            &format!("binPath= \"{bin}\" run"),
            "start= auto",
            &format!("DisplayName= {SERVICE_DISPLAY}"),
        ])
        .status()?;
    if !status.success() {
        bail!("sc create failed");
    }
    let _ = Command::new("sc")
        .args(["description", SERVICE_NAME, "Collects NIC, latency, traffic metrics for Netvan"])
        .status();
    let _ = Command::new("sc")
        .args(["failure", SERVICE_NAME, "reset= 86400", "actions= restart/5000/restart/5000/restart/5000"])
        .status();
    info!("installed {SERVICE_NAME}");
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let _ = stop();
    let status = Command::new("sc").args(["delete", SERVICE_NAME]).status()?;
    if !status.success() {
        bail!("sc delete failed");
    }
    info!("uninstalled {SERVICE_NAME}");
    Ok(())
}

pub fn start() -> Result<()> {
    let status = Command::new("sc").args(["start", SERVICE_NAME]).status()?;
    if !status.success() {
        bail!("sc start failed (may need elevation)");
    }
    Ok(())
}

pub fn stop() -> Result<()> {
    let status = Command::new("sc").args(["stop", SERVICE_NAME]).status()?;
    if !status.success() {
        // ignore if already stopped
    }
    Ok(())
}

#[cfg(windows)]
#[allow(dead_code)]
pub fn run_as_service() -> Result<()> {
    use std::time::Duration;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::{define_windows_service, service_dispatcher};

    define_windows_service!(ffi_service_main, service_main);

    fn service_main(_args: Vec<OsString>) {
        let event_handler = move |control| match control {
            ServiceControl::Stop | ServiceControl::Interrogate => {
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        };
        let status_handle =
            service_control_handler::register(SERVICE_NAME, event_handler).expect("register");
        status_handle
            .set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .ok();
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let _ = rt.block_on(crate::ipc_server::run());
    }

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}
