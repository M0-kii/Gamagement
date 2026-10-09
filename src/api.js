(() => {
  const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
  window.appAPI = {
    addConsole: (data) => invoke("add_console", { data }),
    getConsoles: () => invoke("get_consoles"),
    updateConsole: (data) => invoke("update_console", { data }),
    deleteConsole: (id) => invoke("delete_console", { id }),
    getConsole: (id) => invoke("get_console", { id }),
    startSession: (data) => invoke("start_session", { data }),
    pauseSession: (id) => invoke("pause_session", { id }),
    resumeSession: (id) => invoke("resume_session", { id }),
    endSession: (id) => invoke("end_session", { id }),
    getSessionsForConsole: (consoleId) => invoke("get_sessions_for_console", { consoleId }),
    getSession: (id) => invoke("get_session", { id }),
    addShopItemToSession: (data) => invoke("add_shop_item_to_session", { data }),
    updateSessionControllers: (data) => invoke("update_session_controllers", { data }),
    updateShopItem: (data) => invoke("update_shop_item", { data }),
    deleteShopItem: (data) => invoke("delete_shop_item", { data }),
    confirm: (message) => invoke("ask_confirmation", { message }),
    alert: (message) => invoke("show_message", { message }),
  };
  window.addEventListener("unhandledrejection", (event) => {
    event.preventDefault();
    console.error(event.reason);
    window.appAPI.alert(String(event.reason)).catch(console.error);
  });
})();
