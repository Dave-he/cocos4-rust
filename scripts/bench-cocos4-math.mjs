const iterations = Number.parseInt(process.env.ITERATIONS || "1000000", 10);
const matIterations = Math.max(1, Math.floor(iterations / 10));

class Vec3 {
  constructor(x = 0, y = 0, z = 0) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  static add(out, a, b) {
    out.x = a.x + b.x;
    out.y = a.y + b.y;
    out.z = a.z + b.z;
    return out;
  }

  static cross(out, a, b) {
    const x = a.y * b.z - a.z * b.y;
    const y = a.z * b.x - a.x * b.z;
    const z = a.x * b.y - a.y * b.x;
    out.x = x;
    out.y = y;
    out.z = z;
    return out;
  }

  static dot(a, b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
  }

  static normalize(out, a) {
    const x = a.x;
    const y = a.y;
    const z = a.z;
    let len = x * x + y * y + z * z;
    if (len > 0) {
      len = 1 / Math.sqrt(len);
      out.x = x * len;
      out.y = y * len;
      out.z = z * len;
    }
    return out;
  }

  static transformMat4(out, a, m) {
    const x = a.x;
    const y = a.y;
    const z = a.z;
    const w = m[3] * x + m[7] * y + m[11] * z + m[15];
    const iw = Math.abs(w) > 0.000001 ? 1 / w : 1;
    out.x = (m[0] * x + m[4] * y + m[8] * z + m[12]) * iw;
    out.y = (m[1] * x + m[5] * y + m[9] * z + m[13]) * iw;
    out.z = (m[2] * x + m[6] * y + m[10] * z + m[14]) * iw;
    return out;
  }
}

function multiplyMat4(out, a, b) {
  const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
  const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
  const a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];

  let b0 = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
  out[0] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[1] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[2] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[3] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0 = b[4]; b1 = b[5]; b2 = b[6]; b3 = b[7];
  out[4] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[5] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[6] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[7] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0 = b[8]; b1 = b[9]; b2 = b[10]; b3 = b[11];
  out[8] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[9] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[10] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[11] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;

  b0 = b[12]; b1 = b[13]; b2 = b[14]; b3 = b[15];
  out[12] = b0 * a00 + b1 * a10 + b2 * a20 + b3 * a30;
  out[13] = b0 * a01 + b1 * a11 + b2 * a21 + b3 * a31;
  out[14] = b0 * a02 + b1 * a12 + b2 * a22 + b3 * a32;
  out[15] = b0 * a03 + b1 * a13 + b2 * a23 + b3 * a33;
  return out;
}

function invertMat4(out, a) {
  const a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3];
  const a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7];
  const a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11];
  const a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];
  const b00 = a00 * a11 - a01 * a10;
  const b01 = a00 * a12 - a02 * a10;
  const b02 = a00 * a13 - a03 * a10;
  const b03 = a01 * a12 - a02 * a11;
  const b04 = a01 * a13 - a03 * a11;
  const b05 = a02 * a13 - a03 * a12;
  const b06 = a20 * a31 - a21 * a30;
  const b07 = a20 * a32 - a22 * a30;
  const b08 = a20 * a33 - a23 * a30;
  const b09 = a21 * a32 - a22 * a31;
  const b10 = a21 * a33 - a23 * a31;
  const b11 = a22 * a33 - a23 * a32;
  let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
  if (!det) {
    out.fill(0);
    return out;
  }
  det = 1 / det;
  out[0] = (a11 * b11 - a12 * b10 + a13 * b09) * det;
  out[1] = (a02 * b10 - a01 * b11 - a03 * b09) * det;
  out[2] = (a31 * b05 - a32 * b04 + a33 * b03) * det;
  out[3] = (a22 * b04 - a21 * b05 - a23 * b03) * det;
  out[4] = (a12 * b08 - a10 * b11 - a13 * b07) * det;
  out[5] = (a00 * b11 - a02 * b08 + a03 * b07) * det;
  out[6] = (a32 * b02 - a30 * b05 - a33 * b01) * det;
  out[7] = (a20 * b05 - a22 * b02 + a23 * b01) * det;
  out[8] = (a10 * b10 - a11 * b08 + a13 * b06) * det;
  out[9] = (a01 * b08 - a00 * b10 - a03 * b06) * det;
  out[10] = (a30 * b04 - a31 * b02 + a33 * b00) * det;
  out[11] = (a21 * b02 - a20 * b04 - a23 * b00) * det;
  out[12] = (a11 * b07 - a10 * b09 - a12 * b06) * det;
  out[13] = (a00 * b09 - a01 * b07 + a02 * b06) * det;
  out[14] = (a31 * b01 - a30 * b03 - a32 * b00) * det;
  out[15] = (a20 * b03 - a21 * b01 + a22 * b00) * det;
  return out;
}

function benchVec3(count) {
  const matrix = [1.25, 0.10, 0.05, 0.00, 0.20, 0.90, 0.15, 0.00, 0.05, 0.25, 1.10, 0.00, 4.0, 5.0, 6.0, 1.00];
  const a = new Vec3();
  const b = new Vec3();
  const c = new Vec3();
  const sum = new Vec3();
  const transformed = new Vec3();
  let acc = 0;
  const start = performance.now();
  for (let i = 0; i < count; i += 1) {
    const t = i * 0.001;
    a.x = Math.sin(t) + 1.0;
    a.y = Math.cos(t) + 2.0;
    a.z = t * 0.5 + 3.0;
    b.x = t * 0.25 + 4.0;
    b.y = Math.sin(t) * 2.0 + 5.0;
    b.z = Math.cos(t) * 3.0 + 6.0;
    Vec3.cross(c, a, b);
    Vec3.add(sum, a, b);
    Vec3.add(c, c, sum);
    Vec3.normalize(c, c);
    Vec3.transformMat4(transformed, c, matrix);
    acc += Vec3.dot(transformed, a) + Vec3.dot(a, b);
  }
  return [performance.now() - start, acc];
}

function benchMat4(count) {
  const a = [1.0, 0.2, 0.3, 0.0, 0.1, 1.1, 0.4, 0.0, 0.2, 0.3, 0.9, 0.0, 3.0, 4.0, 5.0, 1.0];
  const b = [0.9, 0.3, 0.1, 0.0, 0.4, 1.0, 0.2, 0.0, 0.1, 0.5, 1.2, 0.0, 6.0, 7.0, 8.0, 1.0];
  const out = new Array(16).fill(0);
  const inv = new Array(16).fill(0);
  let acc = 0;
  const start = performance.now();
  for (let i = 0; i < count; i += 1) {
    multiplyMat4(out, a, b);
    invertMat4(inv, out);
    acc += inv[0] + inv[5] + inv[10] + inv[15];
  }
  return [performance.now() - start, acc];
}

const [vec3Ms, vec3Checksum] = benchVec3(iterations);
const [mat4Ms, mat4Checksum] = benchMat4(matIterations);

console.log("engine=cocos4-js-formula");
console.log(`iterations=${iterations}`);
console.log(`mat4_iterations=${matIterations}`);
console.log(`vec3_hot_path_ms=${vec3Ms.toFixed(3)}`);
console.log(`mat4_hot_path_ms=${mat4Ms.toFixed(3)}`);
console.log(`checksum=${(vec3Checksum + mat4Checksum).toFixed(6)}`);
