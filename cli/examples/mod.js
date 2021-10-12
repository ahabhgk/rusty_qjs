async function main() {
  let a = await Promise.resolve(1);
  let b = await Promise.resolve(2);
  return a + b;
}

main().then(() => {
  throw new Error("haha");
})
