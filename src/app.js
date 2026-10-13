let animator;
let bubble;
let lastBubble = '';
let toolLockUntil = 0;
let idleStart = 0;           // when idle started
let idleAnimTimer = null;    // timer for idle animations

async function init() {
  const resp = await fetch('validation.json');
  const validationData = await resp.json();

  const spriteEl = document.getElementById('sprite');
  const bubbleEl = document.getElementById('bubble');

  animator = new SpriteAnimator(spriteEl, validationData);
  bubble = new Bubble(bubbleEl);

  animator.play('waving');
  bubble.show('爱弥斯已连接 Codex~');

  window._petBubble = bubble;
  window._petAnimator = animator;

  // Drag: use Tauri internals invoke to call Rust start_drag command
  const ipc = window.__TAURI_INTERNALS__;
  document.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return;
    try {
      if (ipc && ipc.invoke) {
        ipc.invoke('start_drag');
      }
    } catch (_) {}
  });

  // Keep the same scale controls available from the keyboard. The tray menu
  // and these shortcuts both call the native scaling implementation.
  document.addEventListener('keydown', (e) => {
    if (!(e.metaKey || e.ctrlKey)) return;
    const scales = { '1': 1, '2': 1.5, '3': 2 };
    const factor = scales[e.key];
    if (!factor) return;
    e.preventDefault();
    try {
      if (ipc && ipc.invoke) {
        ipc.invoke('set_pet_scale', { factor });
      }
    } catch (_) {}
  });

  // Poll HTTP server for state changes
  pollState();
}

function isToolBubble(text) {
  return text && (
    text.includes('正在读取') ||
    text.includes('正在写') ||
    text.includes('正在执行') ||
    text.includes('正在调度') ||
    text.includes('正在搜索') ||
    text.includes('正在获取') ||
    text.includes('正在分析') ||
    text.includes('正在构建') ||
    text.includes('工作中')
  );
}

async function pollState() {
  while (true) {
    try {
      const r = await fetch('http://127.0.0.1:9527/api/current');
      if (r.ok) {
        const data = await r.json();
        if (data.animation) {
          window._petAnimator.play(data.animation);
          // Idle animation: after 15s idle, randomly show different idle poses
          if (data.animation === 'idle') {
            if (!idleStart) idleStart = Date.now();
            scheduleIdleAnim();
          } else {
            idleStart = 0;
            cancelIdleAnim();
          }
        }
        if (data.bubble && data.bubble !== lastBubble) {
          const now = Date.now();
          // If current bubble is a tool bubble and it hasn't been shown long enough, skip update
          if (isToolBubble(lastBubble) && now < toolLockUntil && !isToolBubble(data.bubble)) {
            // keep the tool bubble, skip update
          } else {
            lastBubble = data.bubble;
            window._petBubble.show(data.bubble);
            if (isToolBubble(data.bubble)) {
              toolLockUntil = now + 1200; // lock for 1.2 seconds
            }
          }
        }
      }
    } catch (_) {}
    await new Promise(r => setTimeout(r, 400));
  }
}

function scheduleIdleAnim() {
  if (idleAnimTimer) return;
  const delay = 15000 + Math.random() * 30000; // 15-45 seconds
  idleAnimTimer = setTimeout(doIdleAnim, delay);
}

function doIdleAnim() {
  idleAnimTimer = null;
  if (!idleStart) return;
  // Pick random idle animation
  const anims = ['jumping', 'waving', 'chatting'];
  const pick = anims[Math.floor(Math.random() * anims.length)];
  window._petAnimator.play(pick);
  // Go back to idle after 2 seconds
  setTimeout(() => {
    if (window._petAnimator) window._petAnimator.play('idle');
    scheduleIdleAnim(); // schedule next one
  }, 2000);
}

function cancelIdleAnim() {
  if (idleAnimTimer) {
    clearTimeout(idleAnimTimer);
    idleAnimTimer = null;
  }
}

document.addEventListener('DOMContentLoaded', init);
