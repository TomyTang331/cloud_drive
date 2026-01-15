# Cloud Drive

An open-source cloud storage system built with Rust (Axum backend) and modern frontend technologies.

## ✨ Features

- 📁 File upload, download, delete, and rename
- 🗂️ Folder creation and management
- 🔍 File search and filtering
- 📦 Batch file compression and download
- 👥 User authentication and permission management (JWT)
- 💾 SQLite database storage
- 🎨 Modern frontend interface (Vite + React)

## 🛠️ Tech Stack

### Backend
- **Framework**: Axum 0.7
- **Database**: Sea-ORM + SQLite
- **Authentication**: JWT + bcrypt
- **Logging**: tracing
- **Runtime**: Tokio

### Frontend
- **Build Tool**: Vite
- **Framework**: React
- **Language**: TypeScript

## 🚀 Quick Start

### Requirements
- Rust 1.70+
- Node.js 18+

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/TomyTang331/cloud_drive.git
cd cloud_drive
```

2. **Run backend**
```bash
cargo run --release
```

3. **Run frontend**
```bash
cd frontend
npm install
npm run dev
```

4. **Access the application**
Open your browser and visit `http://localhost:5173`

## 📝 License

This project uses a **dual licensing** model:

### Open Source License (AGPL-3.0)
For the following use cases, this project is freely available under the **GNU Affero General Public License v3.0**:
- ✅ Personal learning and research
- ✅ Open source projects
- ✅ Non-commercial use

**Important**: If you use this project code to provide network services (SaaS), according to AGPL-3.0 license requirements, you must open source your entire codebase.

### Commercial License
If you want to use this project in the following scenarios:
- 🏢 Commercial products or services
- 🔒 Don't want to open source your code
- 💼 Need technical support and custom development

**Please contact me for a commercial license**.

📧 **Contact**: andresromeralito@gmail.com  
💬 **WeChat/QQ**: tomy1999331 / 573945715

For detailed commercial license terms, please refer to [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md)

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📄 License Files

- [LICENSE](LICENSE) - AGPL-3.0 Open Source License
- [LICENSE-COMMERCIAL.md](LICENSE-COMMERCIAL.md) - Commercial License Information

---

**Developer**: TomyTang331  
**Repository**: https://github.com/TomyTang331/cloud_drive
