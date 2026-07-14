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
            logo: {
                src: './src/assets/logo.svg',
            },
            favicon: '/favicon.svg',
            defaultLocale: 'en',
            locales: {
                en: { label: 'English', lang: 'en' },
                de: { label: 'Deutsch', lang: 'de' },
                ja: { label: '日本語', lang: 'ja' },
            },
            customCss: ['/src/styles/custom.css'],
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
                            label: 'Folder Tree Navigation',
                            translations: {
                                de: 'Ordnerbaum-Navigation',
                                ja: 'フォルダツリーの操作',
                            },
                            slug: 'manuals/folder-tree',
                        },
                        {
                            label: 'Pinned Folders',
                            translations: {
                                de: 'Angepinnte Ordner',
                                ja: 'ピン留めフォルダ',
                            },
                            slug: 'manuals/pinned-folders',
                        },
                        {
                            label: 'Search & Filter',
                            translations: {
                                de: 'Suche & Filter',
                                ja: '検索とフィルター',
                            },
                            slug: 'manuals/search-filter',
                        },
                        {
                            label: 'Undo & Redo',
                            translations: {
                                de: 'Rückgängig & Wiederholen',
                                ja: '取り消しとやり直し',
                            },
                            slug: 'manuals/undo-redo',
                        },
                        {
                            label: 'Metadata Panel',
                            translations: {
                                de: 'Metadaten-Panel',
                                ja: 'メタデータパネル',
                            },
                            slug: 'manuals/metadata-panel',
                        },
                        {
                            label: 'Media Playback & Controls',
                            translations: {
                                de: 'Medienwiedergabe & Steuerung',
                                ja: 'メディア再生と操作',
                            },
                            slug: 'manuals/media-playback',
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
                                de: 'Aus dem Quellcode bauen',
                                ja: 'ソースからのビルド',
                            },
                            slug: 'advanced/building-from-source',
                        },
                        {
                            label: 'Configuration File',
                            translations: {
                                de: 'Konfigurationsdatei',
                                ja: '設定ファイルの詳細',
                            },
                            slug: 'advanced/config-file',
                        },
                    ],
                },
                {
                    label: 'Help & Support',
                    translations: {
                        de: 'Hilfe & Support',
                        ja: 'ヘルプとサポート',
                    },
                    items: [
                        {
                            label: 'FAQ / Troubleshooting',
                            translations: {
                                de: 'FAQ & Fehlerbehebung',
                                ja: 'FAQとトラブルシューティング',
                            },
                            slug: 'help/faq',
                        },
                    ],
                },
                {
                    label: 'Contributing',
                    translations: {
                        de: 'Mitwirken',
                        ja: '貢献',
                    },
                    items: [
                        {
                            label: 'Architecture Overview',
                            translations: {
                                de: 'Architekturüberblick',
                                ja: 'アーキテクチャ概要',
                            },
                            slug: 'contributing/architecture',
                        },
                        {
                            label: 'Development Guide',
                            translations: {
                                de: 'Entwicklungsumgebung',
                                ja: '開発ガイド',
                            },
                            slug: 'contributing/development',
                        },
                    ],
                },
                {
                    label: 'About',
                    translations: {
                        de: 'Über',
                        ja: 'について',
                    },
                    items: [
                        {
                            label: 'Privacy Policy',
                            translations: {
                                de: 'Datenschutzerklärung',
                                ja: 'プライバシーポリシー',
                            },
                            slug: 'about/privacy',
                        },
                    ],
                },
            ],
        }),
        react(),
        mdx(),
    ],
});