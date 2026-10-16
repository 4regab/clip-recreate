# Rotoscope pass: trace every reference frame into stacked vector colour layers.
# One fixed palette per shot (so colours do not flicker), nearest-colour labels per frame,
# then each colour region becomes an SVG path. Output: OUTDIR/f####.txt ("#rrggbb\td" lines).
# Path data is in half-pixel units (draw inside scale(0.5)) with relative integer moves.
# usage (from a folder holding frames/f0000.png..f0619.png of the reference clip):
#   python vectorize.py assets/roto 64
import cv2, numpy as np, sys, os
from multiprocessing import Pool

SHOTS = [(0, 1), (1, 40), (40, 87), (87, 102), (102, 105), (105, 122), (122, 165), (165, 180),
         (180, 198), (198, 222), (222, 251), (251, 290), (290, 308), (308, 330), (330, 353),
         (353, 369), (369, 399), (399, 447), (447, 450), (450, 463), (463, 541), (541, 548), (548, 620)]
OUT = sys.argv[1]
K = int(sys.argv[2]) if len(sys.argv) > 2 else 64
UP = 2


def load(i):
    return cv2.bilateralFilter(cv2.imread('frames/f%04d.png' % i), 5, 20, 5)


def palette(a, b):
    data = []
    for i in range(a, b, max(1, (b - a) // 8)):
        data.append(cv2.cvtColor(load(i), cv2.COLOR_BGR2LAB)[::3, ::3].reshape(-1, 3))
    data = np.concatenate(data).astype(np.float32)
    k = min(K, len(np.unique(data[::5], axis=0)))
    cv2.setRNGSeed(7)
    _, _, cen = cv2.kmeans(data, k, None, (cv2.TERM_CRITERIA_EPS + cv2.TERM_CRITERIA_MAX_ITER, 60, 0.1), 2, cv2.KMEANS_PP_CENTERS)
    # nearest-centre lookup table over Lab quantised to 4 units per channel
    g = np.arange(2, 256, 4, dtype=np.float32)
    grid = np.stack(np.meshgrid(g, g, g, indexing='ij'), -1).reshape(-1, 3)
    lut = np.empty(len(grid), np.uint8)
    for s in range(0, len(grid), 16384):
        lut[s:s + 16384] = ((grid[s:s + 16384, None, :] - cen[None]) ** 2).sum(-1).argmin(1)
    return cen, lut


def mask_to_path(m):
    big = cv2.resize(m, (m.shape[1] * UP, m.shape[0] * UP), interpolation=cv2.INTER_LINEAR)
    big = cv2.GaussianBlur(big, (0, 0), 0.8 * UP)
    cs, _ = cv2.findContours((big > 127).astype(np.uint8), cv2.RETR_CCOMP, cv2.CHAIN_APPROX_NONE)
    parts = []
    for c in cs:
        if cv2.contourArea(c) < 3 * UP * UP:
            continue
        c = cv2.approxPolyDP(c, 0.4 * UP, True)
        if len(c) < 3:
            continue
        pts = c[:, 0, :].astype(np.int64)
        rel = np.diff(pts, axis=0)
        parts.append(f'M{pts[0][0]} {pts[0][1]}l' + ' '.join(f'{p[0]} {p[1]}' for p in rel) + 'z')
    return ''.join(parts)


def work(args):
    i, cen, lut = args
    lab = cv2.cvtColor(load(i), cv2.COLOR_BGR2LAB).reshape(-1, 3).astype(np.int32) // 4
    labels = lut[(lab[:, 0] * 64 + lab[:, 1]) * 64 + lab[:, 2]].reshape(720, 960)
    labels = cv2.medianBlur(labels, 3)
    rgb = cv2.cvtColor(cen.reshape(1, -1, 3).astype(np.uint8), cv2.COLOR_LAB2BGR).reshape(-1, 3)
    counts = np.bincount(labels.reshape(-1), minlength=len(cen))
    order = [k for k in np.argsort(-counts) if counts[k] > 0]
    ker = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (3, 3))
    lines = []
    for n, k in enumerate(order):
        d = 'M0 0H1920V1440H0Z' if n == 0 else mask_to_path(cv2.dilate((labels == k).astype(np.uint8) * 255, ker))
        if d:
            c = rgb[k]
            lines.append(f'#{c[2]:02x}{c[1]:02x}{c[0]:02x}\t{d}')
    open(os.path.join(OUT, 'f%04d.txt' % i), 'w').write('\n'.join(lines) + '\n')
    return i


if __name__ == '__main__':
    os.makedirs(OUT, exist_ok=True)
    first, last = (int(sys.argv[3]), int(sys.argv[4])) if len(sys.argv) > 4 else (0, 620)
    jobs = []
    for a, b in SHOTS:
        if b <= first or a >= last:
            continue
        cen, lut = palette(a, b)
        jobs += [(i, cen, lut) for i in range(max(a, first), min(b, last))]
    with Pool(7) as p:
        for n, i in enumerate(p.imap_unordered(work, jobs)):
            if n % 100 == 0:
                print('done', n, '/', len(jobs), flush=True)
    print('finished', len(jobs))
