import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Git-Iris',
  description: 'AI-powered Git workflows, beautifully crafted',
  base: '/git-iris/',

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#e135ff' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Git-Iris Documentation' }],
    ['meta', { property: 'og:description', content: 'AI-powered Git workflows, beautifully crafted' }],
  ],

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Guide', link: '/getting-started/' },
      { text: 'Studio', link: '/studio/' },
      { text: 'Themes', link: '/themes/' },
      { text: 'Architecture', link: '/architecture/' },
      { text: 'Extending', link: '/extending/' },
      {
        text: 'Reference',
        items: [
          { text: 'CLI Reference', link: '/reference/cli' },
          { text: 'Keybindings', link: '/reference/keybindings' },
          { text: 'Tokens', link: '/reference/tokens' },
          { text: 'Troubleshooting', link: '/reference/troubleshooting' },
        ]
      }
    ],

    sidebar: {
      '/getting-started/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Introduction', link: '/getting-started/' },
            { text: 'Installation', link: '/getting-started/installation' },
            { text: 'Quick Start', link: '/getting-started/quick-start' },
            { text: 'Configuration', link: '/getting-started/configuration' },
          ]
        }
      ],
      '/user-guide/': [
        {
          text: 'User Guide',
          items: [
            { text: 'Overview', link: '/user-guide/' },
            { text: 'Commit Messages', link: '/user-guide/commits' },
            { text: 'Code Reviews', link: '/user-guide/reviews' },
            { text: 'Pull Requests', link: '/user-guide/pull-requests' },
            { text: 'Changelogs', link: '/user-guide/changelogs' },
            { text: 'Release Notes', link: '/user-guide/release-notes' },
            { text: 'Presets & Instructions', link: '/user-guide/presets' },
          ]
        }
      ],
      '/studio/': [
        {
          text: 'Iris Studio',
          items: [
            { text: 'Overview', link: '/studio/' },
            { text: 'Navigation', link: '/studio/navigation' },
            { text: 'Chat with Iris', link: '/studio/chat' },
          ]
        },
        {
          text: 'Modes',
          items: [
            { text: 'Explore', link: '/studio/modes/explore' },
            { text: 'Commit', link: '/studio/modes/commit' },
            { text: 'Review', link: '/studio/modes/review' },
            { text: 'Pull Request', link: '/studio/modes/pr' },
            { text: 'Changelog', link: '/studio/modes/changelog' },
            { text: 'Release Notes', link: '/studio/modes/release-notes' },
          ]
        }
      ],
      '/configuration/': [
        {
          text: 'Configuration',
          items: [
            { text: 'Overview', link: '/configuration/' },
            { text: 'Providers', link: '/configuration/providers' },
            { text: 'Models', link: '/configuration/models' },
            { text: 'Project Config', link: '/configuration/project-config' },
            { text: 'Environment', link: '/configuration/environment' },
          ]
        }
      ],
      '/themes/': [
        {
          text: 'Theme System',
          items: [
            { text: 'SilkCircuit Design', link: '/themes/' },
            { text: 'Built-in Themes', link: '/themes/gallery' },
            { text: 'Creating Themes', link: '/themes/creating' },
            { text: 'Token Reference', link: '/themes/tokens' },
            { text: 'Styles & Gradients', link: '/themes/styles' },
          ]
        }
      ],
      '/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/architecture/' },
            { text: 'Agent System', link: '/architecture/agent' },
            { text: 'Capabilities', link: '/architecture/capabilities' },
            { text: 'Tools', link: '/architecture/tools' },
            { text: 'Structured Output', link: '/architecture/output' },
            { text: 'Context Strategy', link: '/architecture/context' },
          ]
        }
      ],
      '/studio-internals/': [
        {
          text: 'Studio Internals',
          items: [
            { text: 'Overview', link: '/studio-internals/' },
            { text: 'Reducer Pattern', link: '/studio-internals/reducer' },
            { text: 'Event System', link: '/studio-internals/events' },
            { text: 'Components', link: '/studio-internals/components' },
          ]
        }
      ],
      '/extending/': [
        {
          text: 'Extending Git-Iris',
          items: [
            { text: 'Overview', link: '/extending/' },
            { text: 'Adding Capabilities', link: '/extending/capabilities' },
            { text: 'Adding Tools', link: '/extending/tools' },
            { text: 'Adding Modes', link: '/extending/modes' },
            { text: 'Contributing', link: '/extending/contributing' },
          ]
        }
      ],
      '/reference/': [
        {
          text: 'Reference',
          items: [
            { text: 'CLI Commands', link: '/reference/cli' },
            { text: 'Keybindings', link: '/reference/keybindings' },
            { text: 'Tokens', link: '/reference/tokens' },
            { text: 'Troubleshooting', link: '/reference/troubleshooting' },
          ]
        }
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/hyperb1iss/git-iris' }
    ],

    footer: {
      message: 'Released under the Apache 2.0 License.',
      copyright: 'Copyright © 2024 Stefanie Jane'
    },

    search: {
      provider: 'local'
    }
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'one-dark-pro'
    }
  }
})
