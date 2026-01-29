const { contextBridge, ipcRenderer } = require('electron');
contextBridge.exposeInMainWorld('electronAPI', {
  addConsole: (data) => ipcRenderer.invoke('addConsole', data),
  getConsoles: () => ipcRenderer.invoke('getConsoles'),
  updateConsole: (data) => ipcRenderer.invoke('updateConsole', data),
  deleteConsole: (id) => ipcRenderer.invoke('deleteConsole', id),
  getConsole: (id) => ipcRenderer.invoke('getConsole', id),
  startSession: (data) => ipcRenderer.invoke('startSession', data),
  pauseSession: (id) => ipcRenderer.invoke('pauseSession', id),
  resumeSession: (id) => ipcRenderer.invoke('resumeSession', id),
  endSession: (id) => ipcRenderer.invoke('endSession', id),
  getSessionsForConsole: (consoleId) => ipcRenderer.invoke('getSessionsForConsole', consoleId),
  getSession: (id) => ipcRenderer.invoke('getSession', id),
  addShopItemToSession: (data) => ipcRenderer.invoke('addShopItemToSession', data)
});