import sharp from "sharp";

async function main(): Promise<void> {
  const output = await sharp({
    create: {
      width: 2,
      height: 2,
      channels: 3,
      background: { r: 24, g: 48, b: 96 },
    },
  })
    .png()
    .toBuffer();

  console.info(
    JSON.stringify(
      {
        sharpVersion: sharp.versions.sharp,
        outputBytes: output.byteLength,
      },
      null,
      2,
    ),
  );
}

void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
