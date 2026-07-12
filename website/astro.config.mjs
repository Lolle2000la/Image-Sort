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
                        {
                            label: 'Download & Installation',
                            translations: {
                                de: 'Download & Installation',
                                ja: 'ダウンロードとインストール',
                            },
                            slug: 'getting-started/installation',
                        },
                        {
                            label: 'First Run Onboarding',
                            translations: {
                                de: 'Erste Schritte & Einrichtung',
                                ja: '初回起動時のオンボーディング',
                            },
                            slug: 'getting-started/onboarding',
                        },
                    ],
                },
                {
                    label: 'User Guide',
                    translations: {
                        de: 'Handbuch',
                        ja: 'ユーザーガイド',
                    },
                    items: [
                        {
                            label: 'Keyboard Layout',
                            translations: {
                                de: 'Tastaturschema & Steuerung',
                                ja: 'キーボードレイアウトと操作方法',
                            },
                            slug: 'manuals/keyboard-layout',
                        },
                        {
                            label: 'Media Capabilities',
                            translations: {
                                de: 'Medienkompatibilitäts-Matrix',
                                ja: '対応メディアフォーマット',
                            },
                            slug: 'manuals/media-capabilities',
                        },
                    ],
                },
                {
                    label: 'Settings & Customization',
                    translations: {
                        de: 'Einstellungen & Anpassung',
                        ja: '設定とカスタマイズ',
                    },
                    items: [
                        {
                            label: 'Application Settings',
                            translations: {
                                de: 'Anwendungseinstellungen',
                                ja: 'アプリケーション設定',
                            },
                            slug: 'config/settings',
                        },
                        {
                            label: 'Custom Keybindings',
                            translations: {
                                de: 'Tastaturkürzel',
                                ja: 'カスタムキーバインド',
                            },
                            slug: 'config/keybindings',
                        },
                    ],
                },
                {
                    label: 'Advanced',
                    translations: {
                        de: 'Erweitert',
                        ja: '高度な設定',
                    },
                    items: [
                        {
                            label: 'Building from Source',
                            translations: {
                                de: 'Aus dem Quellcode kompilieren',
                                ja: 'ソースからのビルド',
                            },
                            slug: 'advanced/building-from-source',
                        },
                    ],
                },
            ],
        }),
        react(),
        mdx(),
    ],
});