const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

const sizes = {
  'mdpi': 48,
  'hdpi': 72,
  'xhdpi': 96,
  'xxhdpi': 144,
  'xxxhdpi': 192,
  '512': 512
};

async function generateIcons() {
  console.log('🚀 Launching browser...');
  const browser = await puppeteer.launch();
  const page = await browser.newPage();
  
  // Load your HTML file
  const htmlPath = `file://${path.resolve(__dirname, 'logo.html')}`;
  await page.goto(htmlPath, { waitUntil: 'networkidle0' });
  
  // Create output directory
  const outDir = path.join(__dirname, 'icons');
  if (!fs.existsSync(outDir)) {
    fs.mkdirSync(outDir);
  }
  
  for (const [density, size] of Object.entries(sizes)) {
    console.log(`📸 Generating ${density} (${size}x${size})...`);
    
    // Set viewport to exact size
    await page.setViewport({ 
      width: size, 
      height: size,
      deviceScaleFactor: 1
    });
    
    // Update both body and logo container to match size exactly
    await page.evaluate((size) => {
      // Make body transparent and sized correctly
      document.body.style.margin = '0';
      document.body.style.padding = '0';
      document.body.style.width = `${size}px`;
      document.body.style.height = `${size}px`;
      document.body.style.overflow = 'hidden';
      document.body.style.background = 'transparent'; // KEY: transparent body
      
      // Set container to exact size (both width AND height)
      const container = document.querySelector('.logo-container');
      container.style.width = `${size}px`;
      container.style.height = `${size}px`;
      container.style.minHeight = `${size}px`;
      container.style.minWidth = `${size}px`;
      container.style.maxHeight = `${size}px`;
      container.style.maxWidth = `${size}px`;
      
      // Scale font proportionally
      const text = document.querySelector('.logo-text');
      text.style.fontSize = `${size * 0.36}px`;
      text.style.lineHeight = '1';
    }, size);
    
    // Wait for fonts to load (correct method)
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Take screenshot with transparent background
    await page.screenshot({
      path: path.join(outDir, `icon-${density}.png`),
      omitBackground: true, // This makes areas without content transparent
      clip: {
        x: 0,
        y: 0,
        width: size,
        height: size
      }
    });
    
    console.log(`  ✅ Saved icon-${density}.png (${size}x${size})`);
  }
  
  console.log('🎉 All icons generated!');
  await browser.close();
}

generateIcons().catch(console.error);

