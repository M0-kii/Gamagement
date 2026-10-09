(() => {
  const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
  window.appAPI = {
    addConsole: (data) => invoke("add_console", { data }),
    updateConsole: (data) => invoke("update_console", { data }),
    archiveConsole: (id) => invoke("archive_console", { id }),
    getConsoles: (includeArchived = false) => invoke("get_consoles", { includeArchived }),
    getDashboard: () => invoke("get_dashboard"),
    startSession: (data) => invoke("start_session", { data }),
    pauseSession: (id) => invoke("pause_session", { id }),
    resumeSession: (id) => invoke("resume_session", { id }),
    getInvoice: (id) => invoke("get_invoice", { id }),
    prepareCheckout: (id) => invoke("prepare_checkout", { id }),
    cancelCheckout: (id) => invoke("cancel_checkout", { id }),
    confirmCheckout: (id) => invoke("confirm_checkout", { id }),
    updateSessionControllers: (data) => invoke("update_session_controllers", { data }),
    getProducts: () => invoke("get_products"),
    saveProduct: (data) => invoke("save_product", { data }),
    archiveProduct: (id) => invoke("archive_product", { id }),
    addProductToSession: (data) => invoke("add_product_to_session", { data }),
    updateItemQuantity: (data) => invoke("update_item_quantity", { data }),
    deleteSessionItem: (id) => invoke("delete_session_item", { id }),
    getHistory: (filter) => invoke("get_history", { filter }),
    exportHistory: (filter) => invoke("export_history", { filter }),
    confirm: (message) => invoke("ask_confirmation", { message }),
  };
})();
