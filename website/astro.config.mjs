// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import react from '@astrojs/react';
import mdx from '@astrojs/mdx';

import { repoUrl } from './src/config.ts';

// https://astro.build/config
export default defineConfig({
    integrations: [
        starlight({
            title: 'Media Sort',
            defaultLocale: 'en',
            locales: {
                en: { label: 'English', lang: 'en' },
                de: { label: 'Deutsch', lang: 'de' },
                ja: { label: '日本語', lang: 'ja' },
            },
            social: [
                { icon: 'github', label: 'GitHub', href: repoUrl }
            ],
            components: {
                Header: './src/components/Header.astro',
            },
            sidebar: [
                {
                    label: 'Getting Started',
                    translations: {
                        de: 'Erste Schritte',
                        ja: 'はじめに',
                    },
                    items: [
                        { label: 'Download & Installation', slug: 'getting-started/installation' },
                        { label: 'Building from Source', slug: 'getting-started/building-from-source' },
                        { label: 'First Run Onboarding', slug: 'getting-started/onboarding' },
                    ],
                },
                {
                    label: 'User Guide',
                    translations: {
                        de: 'Handbuch',
                        ja: 'ユーザーガイド',
                    },
                    items: [
                        { label: 'Keyboard Layout', slug: 'manuals/keyboard-layout' },
                        { label: 'Media Capabilities', slug: 'manuals/media-capabilities' },
                    ],
                },
                {
                    label: 'Technical Configuration',
                    translations: {
                        de: 'Technische Konfiguration',
                        ja: '技術設定',
                    },
                    items: [
                        { label: 'Custom Keybindings', slug: 'config/keybindings' },
                        { label: 'Advanced Settings', slug: 'config/advanced' },
                    ],
                },
            ],
        }),
        react(),
        mdx(),
    ],
});