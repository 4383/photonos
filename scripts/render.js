const { chromium } = require('playwright');

(async () => {
  const url = process.argv[2];
  const screenshotPath = process.argv[3]; // facultatif

  if (!url) {
    console.error("❌ No URL provided.");
    process.exit(1);
  }

  const browser = await chromium.launch();
  const page = await browser.newPage();

  await page.goto(url, { waitUntil: 'networkidle' });

  if (screenshotPath) {
    // Determine screenshot type based on file extension
    const fileExtension = screenshotPath.split('.').pop().toLowerCase();
    const type = ['png', 'jpeg', 'jpg', 'webp'].includes(fileExtension) ? fileExtension : 'png';
    
    // Use explicit type instead of relying on automatic detection
    await page.screenshot({ 
      path: screenshotPath, 
      fullPage: true,
      type: type === 'jpg' ? 'jpeg' : type // Convert 'jpg' to 'jpeg' as Playwright expects
    });
  }

  const html = await page.content();
  console.log(html);

  await browser.close();
})();
