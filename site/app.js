import init, {
  payload_text,
  payload_wifi,
  payload_whatsapp,
  payload_upi,
  render_svg,
  render_matrix,
  render_barcode,
} from './pkg/qr_forge.js';

const $ = (id) => document.getElementById(id);
const errorBox = $('error');
const preview = $('preview');
const payloadPreview = $('payload-preview');

let mode = 'text';
let wasmReady = false;

// ---------- content builders ----------

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

// ---------- graphic abstraction (QR square or barcode strip) ----------

function parseBits(raw) {
  const sep = raw.indexOf(';');
  return { width: Number(raw.slice(0, sep)), bits: raw.slice(sep + 1) };
}

function currentGraphic() {
  const dark = $('dark').value;
  const light = $('light').value;
  const margin = Number($('margin').value);
  if (mode === 'barcode') {
    const format = $('barcode-format').value;
    const value = $('barcode-value').value;
    const raw = render_barcode(format, value);
    const { width, bits } = parseBits(raw);
    const quiet = 10;
    const barHeight = 110;
    const labelHeight = format === 'ean13' ? 26 : 18;
    const totalW = width + quiet * 2;
    const totalH = barHeight + labelHeight;
    const label = format === 'ean13' ? value.replace(/[\s-]/g, '') : value.trim();
    return {
      caption: label,
      svg() {
        let rects = '';
        for (let x = 0; x < width; x += 1) {
          if (bits[x] === '1') {
            rects += `<rect x="${x + quiet}" y="0" width="1" height="${barHeight}" fill="${dark}"/>`;
          }
        }
        const text = label
          ? `<text x="${totalW / 2}" y="${barHeight + labelHeight - 6}" text-anchor="middle" font-family="monospace" font-size="14" fill="${dark}">${label}</text>`
          : '';
        return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${totalW} ${totalH}" width="${totalW * 3}" height="${totalH * 3}"><rect width="${totalW}" height="${totalH}" fill="${light}"/>${rects}${text}</svg>`;
      },
      draw(ctx, scale) {
        ctx.fillStyle = light;
        ctx.fillRect(0, 0, totalW * scale, totalH * scale);
        ctx.fillStyle = dark;
        for (let x = 0; x < width; x += 1) {
          if (bits[x] === '1') ctx.fillRect((x + quiet) * scale, 0, scale, barHeight * scale);
        }
        if (label) {
          ctx.fillStyle = dark;
          ctx.font = `${14 * scale}px monospace`;
          ctx.textAlign = 'center';
          ctx.fillText(label, (totalW * scale) / 2, (barHeight + labelHeight - 6) * scale);
        }
        return { w: totalW * scale, h: totalH * scale };
      },
      aspect: totalH / totalW,
      dims: { w: totalW, h: totalH },
      pad: 0,
    };
  }
  const payload = currentPayload();
  if (!payload.trim()) throw new Error('nothing to encode yet');
  return {
    caption: payload,
    svg() {
      return render_svg(payload, $('ecc').value, dark, light);
    },
    draw(ctx, scale, px) {
      const { width, bits } = parseBits(render_matrix(payload, $('ecc').value));
      const total = width + margin * 2;
      const cell = Math.max(1, Math.floor(px / total));
      const size = cell * total;
      ctx.fillStyle = light;
      ctx.fillRect(0, 0, size, size);
      ctx.fillStyle = dark;
      for (let y = 0; y < width; y += 1) {
        for (let x = 0; x < width; x += 1) {
          if (bits[y * width + x] === '1') ctx.fillRect((x + margin) * cell, (y + margin) * cell, cell, cell);
        }
      }
      return { w: size, h: size };
    },
    aspect: 1,
    pad: margin,
  };
}

// ---------- preview ----------

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
  try {
    const g = currentGraphic();
    preview.style.padding = `${g.pad * 6 + 4}px`;
    preview.style.background = $('light').value;
    preview.style.aspectRatio = mode === 'barcode' ? 'auto' : '1';
    preview.innerHTML = g.svg();
    payloadPreview.textContent = g.caption;
    showError(null);
  } catch (err) {
    showError(err && err.message ? err.message : String(err));
  }
}

function renderToCanvas() {
  const g = currentGraphic();
  const px = Number($('png-size').value);
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d');
  if (mode === 'barcode') {
    const scale = Math.max(2, Math.round(px / g.dims.w));
    canvas.width = g.dims.w * scale;
    canvas.height = g.dims.h * scale;
    g.draw(ctx, scale);
  } else {
    const { w } = g.draw(ctx, 0, px);
    canvas.width = w;
    canvas.height = w;
    g.draw(ctx, 0, px);
  }
  return { canvas, caption: g.caption };
}

// ---------- PDF export (minimal writer, no dependencies) ----------

async function canvasToPdfBytes(canvas, caption) {
  const { width, height } = canvas;
  const rgba = canvas.getContext('2d').getImageData(0, 0, width, height).data;
  const rgb = new Uint8Array(width * height * 3);
  for (let i = 0, j = 0; i < rgba.length; i += 4, j += 3) {
    rgb[j] = rgba[i];
    rgb[j + 1] = rgba[i + 1];
    rgb[j + 2] = rgba[i + 2];
  }
  const zipped = new Uint8Array(
    await new Response(
      new Blob([rgb]).stream().pipeThrough(new CompressionStream('deflate')),
    ).arrayBuffer(),
  );

  const pageW = Math.min(500, width);
  const pageH = Math.round((pageW * height) / width) + 60;
  const imgH = pageH - 60;
  const enc = new TextEncoder();
  const parts = [];
  const offsets = [];
  let pos = 0;
  const push = (s) => {
    const b = typeof s === 'string' ? enc.encode(s) : s;
    parts.push(b);
    pos += b.length;
  };
  const obj = (n, body) => {
    offsets[n] = pos;
    push(`${n} 0 obj\n`);
    push(body);
    push('\nendobj\n');
  };

  push('%PDF-1.4\n');
  const safeCaption = caption.slice(0, 120).replace(/[\\()]/g, ' ');
  obj(1, '<< /Type /Catalog /Pages 2 0 R >>');
  obj(2, '<< /Type /Pages /Kids [3 0 R] /Count 1 >>');
  obj(3, `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${pageW} ${pageH}] /Resources << /XObject << /Im0 4 0 R >> /Font << /F1 5 0 R >> >> /Contents 6 0 R >>`);
  offsets[4] = pos;
  push(`4 0 obj\n<< /Type /XObject /Subtype /Image /Width ${width} /Height ${height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length ${zipped.length} >>\nstream\n`);
  push(zipped);
  push('\nendstream\nendobj\n');
  obj(5, '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>');
  obj(6, `<< /Length ${enc.encode(`q ${pageW} 0 0 ${imgH} 0 50 cm /Im0 Do Q\nBT /F1 9 Tf 20 28 Td (QR Forge) Tj ET\nBT /F1 8 Tf 20 16 Td (${safeCaption}) Tj ET\n`).length} >>\nstream\nq ${pageW} 0 0 ${imgH} 0 50 cm /Im0 Do Q\nBT /F1 9 Tf 20 28 Td (QR Forge) Tj ET\nBT /F1 8 Tf 20 16 Td (${safeCaption}) Tj ET\nendstream`);

  const xrefPos = pos;
  push(`xref\n0 7\n0000000000 65535 f \n`);
  for (let n = 1; n <= 6; n += 1) {
    push(`${String(offsets[n]).padStart(10, '0')} 00000 n \n`);
  }
  push(`trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n${xrefPos}\n%%EOF`);

  const out = new Uint8Array(pos);
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}

// ---------- actions ----------

function download(url, filename) {
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
}

function canvasBlob(canvas) {
  return new Promise((resolve) => canvas.toBlob(resolve, 'image/png'));
}

function guard(fn) {
  return async () => {
    try {
      await fn();
    } catch (err) {
      showError(err && err.message ? err.message : String(err));
    }
  };
}

$('download-png').addEventListener('click', guard(() => {
  const { canvas } = renderToCanvas();
  download(canvas.toDataURL('image/png'), mode === 'barcode' ? 'barcode.png' : 'qr-forge.png');
}));

$('download-svg').addEventListener('click', guard(() => {
  const g = currentGraphic();
  const blob = new Blob([g.svg()], { type: 'image/svg+xml' });
  const url = URL.createObjectURL(blob);
  download(url, mode === 'barcode' ? 'barcode.svg' : 'qr-forge.svg');
  URL.revokeObjectURL(url);
}));

$('download-pdf').addEventListener('click', guard(async () => {
  const { canvas, caption } = renderToCanvas();
  const bytes = await canvasToPdfBytes(canvas, caption);
  const blob = new Blob([bytes], { type: 'application/pdf' });
  const url = URL.createObjectURL(blob);
  download(url, mode === 'barcode' ? 'barcode.pdf' : 'qr-forge.pdf');
  URL.revokeObjectURL(url);
}));

$('print-btn').addEventListener('click', guard(() => {
  const g = currentGraphic();
  $('print-area').innerHTML = `${g.svg()}<p>${g.caption.slice(0, 200)}</p>`;
  window.print();
}));

$('share-btn').addEventListener('click', guard(async () => {
  const { canvas, caption } = renderToCanvas();
  const blob = await canvasBlob(canvas);
  const file = new File([blob], mode === 'barcode' ? 'barcode.png' : 'qr-forge.png', { type: 'image/png' });
  if (navigator.canShare && navigator.canShare({ files: [file] })) {
    await navigator.share({ files: [file], title: 'QR Forge', text: caption.slice(0, 120) });
  } else if (navigator.clipboard && window.ClipboardItem) {
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    $('share-btn').textContent = 'Copied image!';
    setTimeout(() => { $('share-btn').textContent = 'Share'; }, 1500);
  } else {
    throw new Error('sharing is not supported by this browser - use Save PNG');
  }
}));

$('copy-png').addEventListener('click', guard(async () => {
  const { canvas } = renderToCanvas();
  const blob = await canvasBlob(canvas);
  await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
  $('copy-png').textContent = 'Copied!';
  setTimeout(() => { $('copy-png').textContent = 'Copy'; }, 1500);
}));

// ---------- tabs and inputs ----------

$('tabs').addEventListener('click', (event) => {
  const button = event.target.closest('button[data-mode]');
  if (!button) return;
  mode = button.dataset.mode;
  document.querySelectorAll('#tabs button').forEach((b) => b.classList.toggle('active', b === button));
  document.querySelectorAll('.mode-fields').forEach((el) => {
    el.classList.toggle('hidden', el.dataset.mode !== mode);
  });
  $('qr-only').classList.toggle('hidden', mode === 'barcode');
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
