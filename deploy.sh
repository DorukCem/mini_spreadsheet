# Create a file that is ready to upload on itch.io
mkdir -p deploy
cargo build --release --target wasm32-unknown-unknown 
cp target/wasm32-unknown-unknown/release/mini_spreadsheet.wasm deploy
cp index.html deploy
cp -r ./fonts deploy
zip -r deploy.zip deploy
rm -rf deploy