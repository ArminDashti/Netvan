Netvan/
├── Cargo.toml                 # Workspace root
├── package.json               # Frontend package
├── vite.config.ts             # Vite + Tailwind
├── index.html                 # App HTML shell
├── install.ps1                # Build/export Netvan.exe to .\Netvan
├── README.md                  # User-facing overview
├── vendor/README.md           # Optional CLI/driver notes
├── installer/README.md        # Service + install notes
├── docs/                      # Project documentation
│   ├── description.md
│   ├── endpoints.md
│   ├── dir-tree.md
│   ├── modules/
│   ├── suggestion/
│   └── potentional-bugs/
├── crates/
│   ├── netvan-core/           # Types, SQLite, IPC protocol
│   ├── netvan-collectors/     # NIC/ping/HTTP/traffic/speedtest
│   └── netvan-service/        # Windows service + named pipe
├── src/                       # React UI
│   ├── App.tsx                # Routes
│   ├── main.tsx               # Entry
│   ├── index.css              # Theme tokens
│   ├── components/            # Shell + shadcn-style UI
│   ├── pages/                 # Dashboard, NICs, Latency, …
│   └── lib/                   # api.ts, utils.ts
└── src-tauri/                 # Tauri host
    ├── src/lib.rs             # App setup + collectors
    ├── src/commands.rs        # rpc / get_data_dir
    ├── capabilities/          # ACL
    └── permissions/           # Custom command allows (TOML only)
