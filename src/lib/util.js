function sign(v) {
  return `${v > 0 ? '+' : ''}${v}`;
}

function clone(obj) {
  return JSON.parse(JSON.stringify(obj));
}

function randChoice(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function slugify(text) {
  return text.toLowerCase().replaceAll(' ', '_');
}

export {sign, clone, randChoice, slugify};
