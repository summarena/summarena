import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'fs';
import { join } from 'path';
import matter from 'gray-matter';

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
