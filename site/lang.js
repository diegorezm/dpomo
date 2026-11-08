// Every page has an English URL and a Portuguese one (pt/), linked by the #lang switch.
// A first visit from a Portuguese browser goes to pt/; using the switch remembers the choice.
(() => {
  const pt = document.documentElement.lang === "pt-BR";
  const link = document.getElementById("lang");
  let saved = null;
  try { saved = localStorage.getItem("dpomo-lang"); } catch {}
  if (!saved && !pt && (navigator.language || "").toLowerCase().startsWith("pt")) {
    location.replace(link.href + location.hash);
    return;
  }
  link.addEventListener("click", () => {
    try { localStorage.setItem("dpomo-lang", pt ? "en" : "pt"); } catch {}
  });
})();
