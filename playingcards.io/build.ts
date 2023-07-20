#!/usr/bin/env -S deno run --no-prompt --allow-read --allow-write
import { BlobReader, TextReader, Uint8ArrayWriter, ZipWriter } from "https://deno.land/x/zipjs@v2.7.20/index.js";

const output = JSON.parse(await Deno.readTextFile("template.json"));
const zipFileWriter = new Uint8ArrayWriter();
const zipWriter = new ZipWriter(zipFileWriter);
await zipWriter.add("widgets.json", new TextReader(JSON.stringify(output)));
for await (const file of Deno.readDir("images")) {
  if (!file.isFile) continue;
  await zipWriter.add(`userassets/${file.name}`, new BlobReader(new Blob([await Deno.readFile(`images/${file.name}`)])));
}
await zipWriter.close();
await Deno.writeFile("output.pcio", await zipFileWriter.getData());
