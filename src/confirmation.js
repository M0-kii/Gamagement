(() => {
  const queue = [];
  let current = null;
  const dialog = document.getElementById("confirmationDialog");
  const accept = document.getElementById("confirmationAccept");
  const cancel = document.getElementById("confirmationCancel");
  function next() {
    if (current || !queue.length) return;
    current = queue.shift();
    const options = current.options;
    document.getElementById("confirmationTitle").textContent = options.title || "تأیید عملیات";
    document.getElementById("confirmationMessage").textContent = current.message;
    accept.textContent = options.confirmLabel || "تأیید";
    cancel.textContent = options.cancelLabel || "انصراف";
    accept.classList.toggle("btn-danger", options.danger !== false);
    dialog.showModal();
    cancel.focus();
  }
  function finish(result) {
    if (!current) return;
    const request = current;
    current = null;
    dialog.close();
    request.resolve(result);
    queueMicrotask(next);
  }
  accept.addEventListener("click", () => finish(true));
  cancel.addEventListener("click", () => finish(false));
  dialog.addEventListener("cancel", (event) => {
    event.preventDefault();
    finish(false);
  });
  dialog.addEventListener("close", () => { if (!dialog.open && current) finish(false); });
  window.confirmation = {
    ask: (message, options = {}) => new Promise((resolve) => {
      queue.push({ message, options, resolve });
      next();
    }),
  };
})();
