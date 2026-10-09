(() => {
  const maximize = document.getElementById("windowMaximize");
  let closing = false;
  async function perform(action) {
    try { return await action(); }
    catch (error) { notify(String(error), true); }
  }
  function updateMaximized(value) {
    const label = value ? "بازگرداندن اندازه پنجره" : "بزرگ کردن پنجره";
    maximize.setAttribute("aria-label", label);
    maximize.title = label;
    maximize.setAttribute("aria-pressed", String(value));
  }
  async function requestClose() {
    if (closing) return;
    closing = true;
    try {
      if (await window.appAPI.confirm("برنامه بسته شود؟ جلسه‌های باز ذخیره می‌مانند و زمان بازی تا توقف یا تسویه ادامه دارد.", {
        title: "بستن برنامه", confirmLabel: "بستن برنامه", danger: false,
      })) await window.appAPI.closeWindow();
    } catch (error) { notify(String(error), true); }
    finally { closing = false; }
  }
  document.getElementById("windowClose").addEventListener("click", requestClose);
  document.getElementById("windowMinimize").addEventListener("click", () => perform(() => window.appAPI.minimizeWindow()));
  maximize.addEventListener("click", () => perform(async () => updateMaximized(await window.appAPI.toggleMaximizeWindow())));
  const drag = document.getElementById("titlebarDrag");
  drag.addEventListener("mousedown", (event) => {
    if (event.button === 0 && event.detail === 1) perform(() => window.appAPI.startWindowDragging());
  });
  drag.addEventListener("dblclick", () => perform(async () => updateMaximized(await window.appAPI.toggleMaximizeWindow())));
  const events = window.__TAURI__?.event;
  if (events) {
    perform(() => events.listen("window-close-requested", requestClose));
    perform(() => events.listen("window-maximized-changed", (event) => updateMaximized(event.payload)));
  }
  perform(async () => updateMaximized(await window.appAPI.isWindowMaximized()));
})();
