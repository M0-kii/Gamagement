# Gamagement 🎮

**Gamagement** is a professional, sleek, and high-performance management solution for Game Centers. Built with Electron and SQLite, it offers real-time session tracking, dynamic pricing, and a modern custom-themed user interface.

![GitHub License](https://img.shields.io/github/license/M0-kii/Gamagement)
![Electron Version](https://img.shields.io/badge/electron-21.0.0-blue)
![Theme](https://img.shields.io/badge/theme-black%20%26%20purple-purple)

## 🌟 Key Features

- **Real-time Session Management**: Start, pause, and resume gaming sessions with a live countdown timer.
- **Dynamic Pricing**: Automatically calculates costs based on playtime and the number of controllers (1-4 handles).
- **Shop & Inventory**: Add snacks, drinks, or other items to a session mid-play. Prices are calculated upon checkout for a clean live view.
- **Modern UI/UX**: Deep Black & Purple aesthetic with smooth animations and a RTL-first design (Persian/Arabic support).
- **Portable & Lightweight**: Built-in SQLite database means no complex server setup is required.

## 🛠 Technology Stack

- **Frontend**: HTML5, CSS3 (Vanilla), JavaScript
- **Backend**: Electron.js, Node.js
- **Database**: SQLite3
- **Design**: Vazirmatn Typography, Custom CSS Variable Theming

## 📂 Project Structure

```text
Gamagement/
├── assets/          # Application assets (icons, fonts)
├── src/             # Source code (HTML, CSS, JS, Preload)
├── main.js          # Electron entry point (re-routed)
├── package.json     # Project configuration and dependencies
└── README.md        # Documentation
```

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v16 or higher)
- npm

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/M0-kii/Gamagement.git
   cd Gamagement
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run the application:
   ```bash
   npm start
   ```

## 📦 Building for Production

To build a portable Windows executable:

```bash
npm run dist
```

The output will be located in the `dist/` folder.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---
Developed with ❤️ by [M0_kii](https://github.com/M0-kii)
