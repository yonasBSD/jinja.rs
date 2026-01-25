@default:
  cargo run --release -- --template fun.j2

update:
  git add -A ; git commit -am 'updates.'
