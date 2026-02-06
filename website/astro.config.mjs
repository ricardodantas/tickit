// @ts-check
import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
  site: 'https://tickit.ricardodantas.me',
  base: '/',
  build: {
    assets: 'assets'
  }
});
