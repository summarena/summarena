// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import tailwind from '@astrojs/tailwind';
import vue from '@astrojs/vue';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
  site: 'https://summarena.com', // Update with your actual domain
  integrations: [
    tailwind({
      applyBaseStyles: false,
    }),
    vue(),
    starlight({
      title: 'Documentation',
      social: {
        github: 'https://github.com/summarena/summarena',
        twitter: 'https://summarena.com/twitter',
        discord: 'https://summarena.com/discord',
      },
      editLink: {
        baseUrl: 'https://github.com/summarena/summarena/edit/main/website/',
      },
      tableOfContents: {
        minHeadingLevel: 2,
        maxHeadingLevel: 4,
      },
      sidebar: [
        {
          label: 'Guides',
          items: [{ label: 'Getting Started', slug: 'docs/guides/getting-started' }],
        },
      ],
    }),
    mdx(),
    sitemap(),
  ],
});
