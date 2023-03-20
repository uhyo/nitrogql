module.exports = function graphQLLoader(...args) {
  const callback = this.async();
  this.async = () => callback;
  import("./index.mjs").then(
    ({ default: loader }) => loader.apply(this, args),
    (err) => callback(err, null)
  );
};
