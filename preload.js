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

  addShopItem: (data) => ipcRenderer.invoke('addShopItem', data),
  getShopItems: () => ipcRenderer.invoke('getShopItems'),
  getShopItem: (id) => ipcRenderer.invoke('getShopItem', id),
  updateShopItem: (data) => ipcRenderer.invoke('updateShopItem', data),
  deleteShopItem: (id) => ipcRenderer.invoke('deleteShopItem', id),
  getItemPrice: (id) => ipcRenderer.invoke('getItemPrice', id),

  addShopItemToSession: (data) => ipcRenderer.invoke('addShopItemToSession', data)
});