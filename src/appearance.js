(() => {
  let theme = "dark";
  try { theme = window.localStorage.getItem("gamagement-theme") || theme; } catch {}
  let collapsed = false;
  try { collapsed = window.localStorage.getItem("gamagement-sidebar") === "collapsed"; } catch {}
  document.documentElement.dataset.sidebar = collapsed ? "collapsed" : "expanded";
  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.documentElement.dataset.theme = theme === "light" ? "light" : "dark";
})();
