import init, {
  payload_text,
  payload_wifi,
  payload_whatsapp,
  payload_upi,
  render_svg,
  render_matrix,
} from './pkg/qr_forge.js';

const $ = (id) => document.getElementById(id);
const errorBox = $('error');
const preview = $('preview');
const payloadPreview = $('payload-preview');

let mode = 'text';
let wasmReady = false;

function currentPayload() {
  switch (mode) {
    case 'wifi':
      return payload_wifi(
        $('wifi-ssid').value,
        $('wifi-password').value,
        $('wifi-security').value,
        $('wifi-hidden').checked,
      );
    case 'whatsapp':
      return payload_whatsapp($('wa-phone').value, $('wa-message').value);
    case 'upi':
      return payload_upi(
        $('upi-vpa').value,
        $('upi-name').value,
        $('upi-amount').value,
        $('upi-note').value,
      );
    default:
      return payload_text($('text-content').value);
  }
}

function showError(message) {
  if (message) {
    errorBox.textContent = message;
    errorBox.hidden = false;
    preview.classList.add('empty');
  } else {
    errorBox.hidden = true;
    preview.classList.remove('empty');
  }
}

function update() {
  if (!wasmReady) return;
  let payload;
  try {
    payload = currentPayload();
    if (!payload.trim()) throw new Error('nothing to encode yet');
    const svg = render_svg(payload, $('ecc').value, $('dark').value, $('light').value);
    const margin = Number($('margin').value);
    preview.style.padding = `${margin * 6 + 4}px`;
    preview.style.background = $('light').value;
    preview.innerHTML = svg;
    payloadPreview.textContent = payload;
    showError(null);
  } catch (err) {
    showError(err && err.message ? err.message : String(err));
    return;
  }
}

function drawCanvas(sizePx) {
  const payload = currentPayload();
  if (!payload.trim()) throw new Error('nothing to encode yet');
  const matrix = render_matrix(payload, $('ecc').value);
  const sep = matrix.indexOf(';');
  const width = Number(matrix.slice(0, sep));
  const bits = matrix.slice(sep + 1);
  const margin = Number($('margin').value);
  const total = width + margin * 2;
  const cell = Math.max(1, Math.floor(sizePx / total));
  const px = cell * total;
  const canvas = document.createElement('canvas');
  canvas.width = px;
  canvas.height = px;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = $('light').value;
  ctx.fillRect(0, 0, px, px);
  ctx.fillStyle = $('dark').value;
  for (let y = 0; y < width; y += 1) {
    for (let x = 0; x < width; x += 1) {
      if (bits[y * width + x] === '1') {
        ctx.fillRect((x + margin) * cell, (y + margin) * cell, cell, cell);
      }
    }
  }
  return canvas;
}

function download(url, filename) {
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
}

$('download-png').addEventListener('click', () => {
  try {
    const canvas = drawCanvas(Number($('png-size').value));
    download(canvas.toDataURL('image/png'), 'qr-forge.png');
  } catch (err) {
    showError(err && err.message ? err.message : String(err));
  }
});

$('download-svg').addEventListener('click', () => {
  try {
    const payload = currentPayload();
    const svg = render_svg(payload, $('ecc').value, $('dark').value, $('light').value);
    const blob = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    download(url, 'qr-forge.svg');
    URL.revokeObjectURL(url);
  } catch (err) {
    showError(err && err.message ? err.message : String(err));
  }
});

$('copy-png').addEventListener('click', async () => {
  try {
    const canvas = drawCanvas(Number($('png-size').value));
    const blob = await new Promise((resolve) => canvas.toBlob(resolve, 'image/png'));
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    $('copy-png').textContent = 'Copied!';
    setTimeout(() => { $('copy-png').textContent = 'Copy'; }, 1500);
  } catch (err) {
    showError('copy needs a secure (HTTPS) page and clipboard permission');
  }
});

$('tabs').addEventListener('click', (event) => {
  const button = event.target.closest('button[data-mode]');
  if (!button) return;
  mode = button.dataset.mode;
  document.querySelectorAll('#tabs button').forEach((b) => b.classList.toggle('active', b === button));
  document.querySelectorAll('.mode-fields').forEach((el) => {
    el.classList.toggle('hidden', el.dataset.mode !== mode);
  });
  update();
});

$('margin').addEventListener('input', () => {
  $('margin-value').textContent = $('margin').value;
});

document.querySelectorAll('input, textarea, select').forEach((el) => {
  el.addEventListener('input', update);
  el.addEventListener('change', update);
});

init().then(() => {
  wasmReady = true;
  update();
}).catch((err) => {
  showError(`failed to load the WebAssembly module: ${err}`);
});
