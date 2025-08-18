import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync, existsSync } from 'fs';
import { join } from 'path';
import matter from 'gray-matter';
import { execSync } from 'child_process';

describe('Blog content validation', () => {
  const blogDir = join(process.cwd(), 'src/content/blog');

  it('should have blog directory', () => {
    expect(() => readdirSync(blogDir)).not.toThrow();
  });

  it('should have valid markdown files with frontmatter', () => {
    const files = readdirSync(blogDir).filter(
      file => file.endsWith('.md') || file.endsWith('.mdx')
    );

    expect(files.length).toBeGreaterThan(0);

    files.forEach(file => {
      const filePath = join(blogDir, file);
      const content = readFileSync(filePath, 'utf-8');
      const parsed = matter(content);

      // Check required frontmatter fields
      expect(parsed.data.title).toBeDefined();
      expect(typeof parsed.data.title).toBe('string');

      expect(parsed.data.description).toBeDefined();
      expect(typeof parsed.data.description).toBe('string');

      expect(parsed.data.pubDate).toBeDefined();

      // Validate date format
      const date = new Date(parsed.data.pubDate);
      expect(date instanceof Date).toBe(true);
      expect(date.toString()).not.toBe('Invalid Date');
    });
  });

  it('should have valid blog post content structure', () => {
    const files = readdirSync(blogDir).filter(
      file => file.endsWith('.md') || file.endsWith('.mdx')
    );

    files.forEach(file => {
      const filePath = join(blogDir, file);
      const content = readFileSync(filePath, 'utf-8');
      const parsed = matter(content);

      // Check that there's actual content
      expect(parsed.content.trim().length).toBeGreaterThan(0);

      // Check for common markdown structure
      expect(parsed.content).toMatch(/\w+/); // Has some words
    });
  });
});

describe('Tailwind CSS Integration', () => {
  it('should generate CSS with Tailwind classes when building', () => {
    // Build the project to ensure CSS is generated
    try {
      execSync('pnpm build', {
        cwd: process.cwd(),
        stdio: 'pipe',
        timeout: 60000, // 60 second timeout
      });
    } catch (error) {
      throw new Error(`Build failed: ${error}`);
    }

    // Check if dist directory exists
    const distDir = join(process.cwd(), 'dist');
    expect(existsSync(distDir)).toBe(true);

    // Find CSS files in the dist directory
    const findCSSFiles = (dir: string): string[] => {
      const files: string[] = [];
      const items = readdirSync(dir);

      for (const item of items) {
        const fullPath = join(dir, item);
        try {
          readFileSync(fullPath, { encoding: 'utf8' });
          if (item.endsWith('.css')) {
            files.push(fullPath);
          }
        } catch {
          // If it's a directory or unreadable file, check if it's a directory
          try {
            readdirSync(fullPath);
            files.push(...findCSSFiles(fullPath));
          } catch {
            // Not a directory, skip
          }
        }
      }
      return files;
    };

    const cssFiles = findCSSFiles(distDir);
    expect(cssFiles.length).toBeGreaterThan(0);

    // Check that at least one CSS file contains Tailwind-generated styles
    let foundTailwindStyles = false;

    for (const cssFile of cssFiles) {
      const cssContent = readFileSync(cssFile, 'utf-8');

      // Check for common Tailwind patterns that indicate it's working
      const tailwindIndicators = [
        // CSS custom properties (Tailwind uses these extensively)
        /--tw-translate-x/,
        /--tw-border-spacing/,
        // Common Tailwind utility patterns
        /\.bg-gray-\d+/,
        /\.text-\d+xl/,
        /\.px-\d+/,
        /\.py-\d+/,
        /\.flex\{display:flex\}/,
        /\.grid\{display:grid\}/,
        // Tailwind base styles
        /\*,:before,:after\{/,
        // Responsive prefixes
        /@media \(min-width: \d+px\)/,
      ];

      const hasMultipleIndicators =
        tailwindIndicators.filter(pattern => pattern.test(cssContent)).length >= 5; // Require at least 5 different Tailwind patterns

      if (hasMultipleIndicators) {
        foundTailwindStyles = true;
        break;
      }
    }

    expect(foundTailwindStyles).toBe(true);
  }, 120000); // 2 minute timeout for this test

  it('should have Tailwind configuration files', () => {
    // Check that Tailwind config exists
    const tailwindConfig = join(process.cwd(), 'tailwind.config.mjs');
    expect(existsSync(tailwindConfig)).toBe(true);

    // Verify the config contains expected Tailwind setup
    const configContent = readFileSync(tailwindConfig, 'utf-8');
    expect(configContent).toContain('tailwindcss');
  });

  it('should include Tailwind CSS classes in HTML output', () => {
    // Check the built HTML contains Tailwind classes
    const distDir = join(process.cwd(), 'dist');

    if (!existsSync(distDir)) {
      // Build if dist doesn't exist
      execSync('pnpm build', {
        cwd: process.cwd(),
        stdio: 'pipe',
        timeout: 60000,
      });
    }

    const indexPath = join(distDir, 'index.html');
    expect(existsSync(indexPath)).toBe(true);

    const htmlContent = readFileSync(indexPath, 'utf-8');

    // Check for common Tailwind utility classes that should be in our HTML
    const commonTailwindClasses = [
      'class=',
      'bg-',
      'text-',
      'flex',
      'grid',
      'p-',
      'm-',
      'w-',
      'h-',
    ];

    const foundClasses = commonTailwindClasses.filter(className => htmlContent.includes(className));

    // Should find multiple Tailwind classes in the HTML
    expect(foundClasses.length).toBeGreaterThan(3);
  });
});
