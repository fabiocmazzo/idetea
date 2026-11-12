/* Local player only. No network calls. */
(function () {
  var root = document.querySelector("[data-demo]");
  if (!root) return;
  var video = root.querySelector("video");
  var button = root.querySelector(".sound");
  if (!video || !button) return;

  var playLabel = button.getAttribute("data-play") || "Play with sound";
  var replayLabel = button.getAttribute("data-replay") || "Replay with sound";
  var reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  function show(label) {
    button.hidden = false;
    button.textContent = label;
  }

  function preview() {
    video.loop = true;
    video.muted = true;
    video.controls = false;
    show(video.ended ? replayLabel : playLabel);
    if (reduce) {
      video.removeAttribute("autoplay");
      video.pause();
      return;
    }
    var pending = video.play();
    if (pending && pending.catch) pending.catch(function () {});
  }

  function withSound() {
    video.loop = false;
    video.muted = false;
    video.controls = true;
    button.hidden = true;
    try { video.currentTime = 0; } catch (e) {}
    var pending = video.play();
    if (pending && pending.catch) {
      pending.catch(function () { show(playLabel); });
    }
  }

  button.addEventListener("click", function (event) {
    event.preventDefault();
    event.stopPropagation();
    withSound();
  });
  video.addEventListener("click", function () {
    if (video.muted) withSound();
  });
  video.addEventListener("ended", function () {
    preview();
    show(replayLabel);
  });

  if (reduce) {
    video.pause();
    video.removeAttribute("autoplay");
    show(playLabel);
  } else {
    preview();
  }
})();
