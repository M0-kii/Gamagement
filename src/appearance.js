(() => {
  let theme = "dark";
  try { theme = window.localStorage.getItem("gamagement-theme") || theme; } catch {}
  document.documentElement.dataset.theme = theme === "light" ? "light" : "dark";
})();
