import { nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from 'vitest'

import ProblemBodyAssets from '../ProblemBodyAssets.vue'
import { problemBodyAssetsFixture } from '../ProblemBodyAssets.fixture'

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

beforeAll(() => {
  vi.stubGlobal('ResizeObserver', ResizeObserverStub)
})

afterAll(() => {
  vi.unstubAllGlobals()
})

afterEach(() => {
  document.body.replaceChildren()
})

describe('ProblemBodyAssets', () => {
  it('renders the public problem fixture body and asset in source order', () => {
    const wrapper = mount(ProblemBodyAssets, { props: problemBodyAssetsFixture })
    const fixtureAsset = problemBodyAssetsFixture.assets[0]

    expect(fixtureAsset).toBeDefined()
    expect(wrapper.get('[data-testid="problem-markdown"]').text()).toBe(
      problemBodyAssetsFixture.bodyMarkdown,
    )
    expect(wrapper.findAll('img')).toHaveLength(problemBodyAssetsFixture.assets.length)
    expect(wrapper.get('img').attributes('src')).toBe(fixtureAsset?.url)
    expect(wrapper.get('img').attributes('alt')).toBe(fixtureAsset?.alt)
  })

  it('renders supported Markdown semantics and normalizes a level-one heading', () => {
    const wrapper = mount(ProblemBodyAssets, {
      props: {
        bodyMarkdown: `# 見出し

**重要**な項目

- ひとつ
- ふたつ

| キー | 動作 |
| --- | --- |
| A | 移動 |`,
        assets: [],
      },
    })

    expect(wrapper.find('h1').exists()).toBe(false)
    expect(wrapper.get('h2').text()).toBe('見出し')
    expect(wrapper.get('strong').text()).toBe('重要')
    expect(wrapper.findAll('li').map((item) => item.text())).toEqual(['ひとつ', 'ふたつ'])
    expect(wrapper.get('table').text()).toContain('移動')
  })

  it('does not interpret raw HTML, executable links, or Markdown images', () => {
    const wrapper = mount(ProblemBodyAssets, {
      props: {
        bodyMarkdown: `<script>window.exposed = true</script>

<img src="https://example.com/raw.png" onerror="window.exposed = true">

[危険なリンク](javascript:alert(1))

![Markdown画像](https://example.com/markdown.png)`,
        assets: [],
      },
    })

    expect(wrapper.find('script').exists()).toBe(false)
    expect(wrapper.find('img').exists()).toBe(false)
    expect(wrapper.find('[onerror]').exists()).toBe(false)
    expect(wrapper.findAll('a')).toHaveLength(0)
  })

  it('allows relative and HTTPS links while rejecting other schemes', () => {
    const wrapper = mount(ProblemBodyAssets, {
      props: {
        bodyMarkdown:
          '[相対リンク](/rules) [HTTPS](https://example.com/rules) [HTTP](http://example.com/rules) [data](data:text/html,test)',
        assets: [],
      },
    })

    expect(wrapper.findAll('a').map((link) => link.attributes('href'))).toEqual([
      '/rules',
      'https://example.com/rules',
    ])
    expect(
      wrapper.findAll('a').every((link) => link.attributes('referrerpolicy') === 'no-referrer'),
    ).toBe(true)
  })

  it('fetches only image assets with relative or HTTPS URLs', () => {
    const wrapper = mount(ProblemBodyAssets, {
      props: {
        bodyMarkdown: '問題文',
        assets: [
          { type: 'image', url: '/safe.png', alt: '安全な画像' },
          { type: 'image', url: 'http://example.com/insecure.png', alt: 'HTTP画像' },
          { type: 'image', url: 'javascript:alert(1)', alt: '危険な画像' },
          { type: 'image', url: '/\\evil.example/tracker.png', alt: '偽の相対URL' },
          { type: 'video', url: 'https://example.com/movie.mp4', alt: '動画' },
        ],
      },
    })

    expect(wrapper.findAll('img')).toHaveLength(1)
    expect(wrapper.get('img').attributes()).toMatchObject({
      src: '/safe.png',
      alt: '安全な画像',
      referrerpolicy: 'no-referrer',
    })
    expect(wrapper.findAll('[data-testid="asset-fallback"]')).toHaveLength(4)
    expect(wrapper.html()).not.toContain('javascript:alert(1)')
    expect(wrapper.html()).not.toContain('http://example.com/insecure.png')
  })

  it('shows an accessible fallback when an image cannot be loaded', async () => {
    const wrapper = mount(ProblemBodyAssets, {
      props: {
        bodyMarkdown: '問題文',
        assets: [{ type: 'image', url: '/missing.png', alt: '迷路の図' }],
      },
    })

    await wrapper.get('img').trigger('error')

    expect(wrapper.find('img').exists()).toBe(false)
    expect(wrapper.get('[data-testid="asset-fallback"]').text()).toContain(
      '画像を読み込めませんでした',
    )
    expect(wrapper.get('[data-testid="asset-fallback"]').attributes('role')).toBe('status')
    expect(wrapper.get('[data-testid="asset-fallback"]').text()).toContain('迷路の図')
  })

  it('opens the selected image in a Headless UI dialog and closes it', async () => {
    const wrapper = mount(ProblemBodyAssets, {
      attachTo: document.body,
      props: {
        bodyMarkdown: '問題文',
        assets: [{ type: 'image', url: '/map.png', alt: '地図' }],
      },
    })

    await wrapper.get('button[aria-label="画像を拡大: 地図"]').trigger('click')
    await nextTick()

    const dialog = document.body.querySelector<HTMLElement>('[role="dialog"]')
    expect(dialog).not.toBeNull()
    expect(dialog?.textContent).toContain('地図')
    expect(dialog?.querySelector('img')?.getAttribute('src')).toBe('/map.png')
    expect(dialog?.querySelector('img')?.getAttribute('alt')).toBe('')

    const closeButton = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === '閉じる',
    )
    expect(closeButton).toBeDefined()
    closeButton?.click()
    await nextTick()

    expect(document.body.querySelector('[role="dialog"]')).toBeNull()
    wrapper.unmount()
  })

  it('does not inherit unknown private fields into the DOM', () => {
    const secretAnswer = 'PRIVATE_ANSWER_SENTINEL'
    const secretJudgeConfig = 'PRIVATE_JUDGE_SENTINEL'
    const wrapper = mount(ProblemBodyAssets, {
      props: problemBodyAssetsFixture,
      attrs: {
        answer: secretAnswer,
        judge_config: secretJudgeConfig,
      },
    })

    expect(wrapper.html()).not.toContain(secretAnswer)
    expect(wrapper.html()).not.toContain(secretJudgeConfig)
    expect(wrapper.attributes('answer')).toBeUndefined()
    expect(wrapper.attributes('judge_config')).toBeUndefined()
  })

  it('updates its public content and never leaves stale expanded asset data', async () => {
    const wrapper = mount(ProblemBodyAssets, {
      attachTo: document.body,
      props: {
        bodyMarkdown: '最初の問題',
        assets: [{ type: 'image', url: '/first.png', alt: '最初の図' }],
      },
    })

    await wrapper.get('button').trigger('click')
    await wrapper.setProps({
      bodyMarkdown: '**次の問題**',
      assets: [{ type: 'image', url: '/first.png', alt: '更新された図' }],
    })
    await nextTick()

    expect(document.body.querySelector('[role="dialog"]')?.textContent).toContain('更新された図')

    await wrapper.setProps({
      assets: [],
    })
    await nextTick()

    expect(wrapper.get('[data-testid="problem-markdown"] strong').text()).toBe('次の問題')
    expect(wrapper.find('section[aria-label="問題資料"]').exists()).toBe(false)
    expect(document.body.querySelector('[role="dialog"]')).toBeNull()
    wrapper.unmount()
  })

  it('closes the dialog if an updated asset is no longer safe to render', async () => {
    const unsafeUrl = 'http://evil.example/tracker.png'
    const wrapper = mount(ProblemBodyAssets, {
      attachTo: document.body,
      props: {
        bodyMarkdown: '問題文',
        assets: [
          {
            type: 'image',
            url: `https://safe.example/:${unsafeUrl}`,
            alt: '安全な画像',
          },
        ],
      },
    })

    await wrapper.get('button').trigger('click')
    expect(document.body.querySelector('[role="dialog"]')).not.toBeNull()

    await wrapper.setProps({
      assets: [
        {
          type: 'image:https://safe.example/',
          url: unsafeUrl,
          alt: '表示してはいけない画像',
        },
      ],
    })
    await nextTick()

    expect(document.body.querySelector('[role="dialog"]')).toBeNull()
    expect(document.body.querySelector(`img[src="${unsafeUrl}"]`)).toBeNull()
    wrapper.unmount()
  })
})
