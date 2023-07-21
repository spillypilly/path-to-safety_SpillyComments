#!/usr/bin/env -S deno run --no-prompt --allow-read --allow-write
import { BlobReader, TextReader, Uint8ArrayWriter, ZipWriter } from "https://deno.land/x/zipjs@v2.7.20/index.js";
import { range, zip } from "https://deno.land/x/lodash@4.17.15-es/lodash.js";
// @deno-types="npm:@types/react@18.2.15"
import React, { ReactNode } from "npm:react@18.2.0";
import ReactMarkdown from "npm:react-markdown@8.0.7";
import satori from "npm:satori@0.10.1";

function Rules({ page, numPages, content }: { page: number; numPages: number; content: string }) {
  const markdown = ReactMarkdown({ components: markdownComponents, children: content });
  return (
    <div tw="flex flex-col items-center w-full h-full bg-white font-sans text-base p-1">
      <div tw="flex flex-col flex-grow w-full">
        <div tw="flex flex-col" {...markdown.props} />
      </div>
      <div tw="flex text-center pt-0.5">
        Page {page} of {numPages}
      </div>
    </div>
  );
}

const pages = (await Deno.readTextFile("rules.md")).split("---");
for (const [page, pageNum] of enumerate(pages)) {
  render(`images/rules_${pageNum + 1}.svg`, <Rules page={pageNum + 1} numPages={pages.length} content={page} />);
}

const markdownComponents = {
  h1: ({ node, ...props }: Record<string, unknown>) => <div tw="flex text-xl font-bold mb-0.5" {...props} />,
  h3: ({ node, ...props }: Record<string, unknown>) => <div tw="flex text-lg font-bold mb-0.5" {...props} />,
  h4: ({ node, ...props }: Record<string, unknown>) => <div tw="flex font-bold" {...props} />,
  p: ({ node, ...props }: Record<string, unknown>) => <div tw="flex mb-0.5" {...props} />,
  ol: ({ node, ...props }: Record<string, unknown>) => <div tw="flex flex-col" {...props} />,
  li: ({ node, ...props }: Record<string, unknown>) => <div tw="flex mb-0.5">• {props.children as ReactNode}</div>,
};

function enumerate<T>(input: Array<T>): Array<[T, number]> {
  return zip(input, range(input.length));
}

async function render(path: string, node: ReactNode) {
  await Deno.writeTextFile(
    path,
    await satori(node, {
      width: 103,
      height: 160,
      embedFont: false,
      fonts: [
        {
          name: "Verdana",
          data: await Deno.readFile("fonts/verdana.woff"),
          weight: 400,
          style: "normal",
        },
        {
          name: "Verdana",
          data: await Deno.readFile("fonts/verdana-bold.woff"),
          weight: 800,
          style: "normal",
        },
      ],
      tailwindConfig: {
        theme: {
          extend: {
            fontFamily: {
              sans: "Verdana",
            },
            fontSize: {
              base: "4.5px",
              lg: "5.5px",
              xl: "6px",
            },
          },
        },
      },
    }),
  );
}

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
