(() => {
  const calendar = new Intl.DateTimeFormat("en-US-u-ca-persian", {
    timeZone: "Asia/Tehran", year: "numeric", month: "2-digit", day: "2-digit",
  });
  const digits = (value) => String(value).replace(/[۰-۹]/g, (c) => "۰۱۲۳۴۵۶۷۸۹".indexOf(c))
    .replace(/[٠-٩]/g, (c) => "٠١٢٣٤٥٦٧٨٩".indexOf(c));
  function jalaliParts(date) {
    const parts = Object.fromEntries(calendar.formatToParts(date).map((p) => [p.type, p.value]));
    return [Number(parts.year), Number(parts.month), Number(parts.day)];
  }
  function jalaliDate(date = new Date()) {
    return jalaliParts(date).map((n, i) => String(n).padStart(i ? 2 : 4, "0")).join("/");
  }
  function dateBoundary(value, nextDay = false) {
    if (!value.trim()) return null;
    const match = digits(value.trim()).match(/^(\d{4})[\/\-](\d{1,2})[\/\-](\d{1,2})$/);
    if (!match) throw new Error("تاریخ را به صورت ۱۴۰۵/۰۷/۱۷ وارد کنید");
    const wanted = match.slice(1).map(Number);
    if (wanted[0] < 1300 || wanted[0] > 1600 || wanted[1] < 1 || wanted[1] > 12 || wanted[2] < 1 || wanted[2] > 31) {
      throw new Error("تاریخ شمسی نامعتبر است");
    }
    const key = ([y, m, d]) => y * 10000 + m * 100 + d;
    let low = Math.floor(Date.UTC(wanted[0] + 621, 0, 1) / 86400000);
    let high = Math.floor(Date.UTC(wanted[0] + 623, 0, 1) / 86400000);
    while (low <= high) {
      const day = Math.floor((low + high) / 2);
      const actual = key(jalaliParts(new Date(day * 86400000)));
      if (actual === key(wanted)) {
        return new Date((day + Number(nextDay)) * 86400000 - 210 * 60000).toISOString();
      }
      if (actual < key(wanted)) low = day + 1;
      else high = day - 1;
    }
    throw new Error("این روز در تقویم شمسی وجود ندارد");
  }
  const number = new Intl.NumberFormat("fa-IR");
  const dateTime = new Intl.DateTimeFormat("fa-IR", {
    timeZone: "Asia/Tehran", dateStyle: "short", timeStyle: "short",
  });
  window.formatters = {
    digits, jalaliDate, dateBoundary,
    price: (value) => number.format(value) + " تومان",
    number: (value) => number.format(value),
    date: (value) => value ? dateTime.format(new Date(value)) : "—",
    duration: (ms) => {
      const seconds = Math.floor(Math.max(0, ms) / 1000);
      return [Math.floor(seconds / 3600), Math.floor(seconds / 60) % 60, seconds % 60]
        .map((n) => String(n).padStart(2, "0")).join(":");
    },
    escape: (value) => String(value).replace(/[&<>"']/g, (c) => ({
      "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
    })[c]),
  };
})();
