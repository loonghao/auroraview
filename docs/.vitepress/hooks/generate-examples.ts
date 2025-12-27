/**
 * Generate examples documentation from examples/ directory.
 *
 * This script is called during VitePress build to generate
 * static markdown files from Python examples.
 */

import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const EXAMPLES_DIR = path.resolve(__dirname, '../../../examples')
const DOCS_DIR = path.resolve(__dirname, '../..')
const SCREENSHOTS_DIR = path.resolve(DOCS_DIR, 'public/examples')

interface ExampleMeta {
  name: string
  title: string
  description: string
  category: string
  features: string[]
  skip: boolean
  skipReason?: string
  hasScreenshot: boolean
  source: string
}

// Category definitions and order
const CATEGORIES: Record<string, string[]> = {
  'Getting Started': [
    'simple_decorator',
    'dynamic_binding',
  ],
  'Window Features': [
    'window_effects_demo',
    'window_events_demo',
    'floating_panel_demo',
    'multi_window_demo',
    'child_window_demo',
    'child_aware_demo',
  ],
  'UI Components': [
    'native_menu_demo',
    'custom_context_menu_demo',
    'system_tray_demo',
    'logo_button_demo',
  ],
  'Advanced Patterns': [
    'desktop_app_demo',
    'desktop_events_demo',
    'signals_advanced_demo',
    'cookie_management_demo',
    'dom_manipulation_demo',
    'ipc_channel_demo',
    'local_assets_example',
  ],
  'DCC Integration': [
    'qt_style_tool',
    'qt_custom_menu_demo',
    'maya_qt_echo_demo',
    'dcc_integration_example',
  ],
}

const SKIP_REASONS: Record<string, string> = {
  'qt_style_tool': 'Requires Qt/PySide',
  'qt_custom_menu_demo': 'Requires Qt/PySide',
  'maya_qt_echo_demo': 'Requires Maya',
  'dcc_integration_example': 'Requires DCC application',
}

function parseExampleMeta(filePath: string): ExampleMeta | null {
  const name = path.basename(filePath, '.py')

  if (name.startsWith('__') || name === 'pack-example') {
    return null
  }

  let content: string
  try {
    content = fs.readFileSync(filePath, 'utf-8')
  } catch {
    return null
  }

  const docstringMatch = content.match(/^"""([\s\S]*?)"""/m) ||
                         content.match(/^'''([\s\S]*?)'''/m) ||
                         content.match(/^#!.*\n(?:#.*\n)*"""([\s\S]*?)"""/m)

  let title = name.replace(/_/g, ' ').replace(/demo$/i, '').trim()
  title = title.charAt(0).toUpperCase() + title.slice(1)
  let description = ''
  const features: string[] = []

  if (docstringMatch) {
    const docstring = docstringMatch[1].trim()
    const lines = docstring.split('\n')

    if (lines.length > 0) {
      const firstLine = lines[0].trim()
      if (firstLine && !firstLine.startsWith('-')) {
        const titleMatch = firstLine.match(/^([^.-]+)/)
        if (titleMatch) {
          title = titleMatch[1].trim()
        }
      }
    }

    const descLines: string[] = []
    let inFeatures = false
    let inUsage = false

    for (let i = 1; i < lines.length; i++) {
      const line = lines[i]
      const trimmed = line.trim()

      if (trimmed.toLowerCase().startsWith('features')) {
        inFeatures = true
        inUsage = false
        continue
      }
      if (trimmed.toLowerCase().startsWith('usage:') ||
          trimmed.toLowerCase().startsWith('usage ')) {
        inUsage = true
        inFeatures = false
        continue
      }
      if (trimmed.toLowerCase().startsWith('platform') ||
          trimmed.toLowerCase().startsWith('signed-off')) {
        break
      }

      if (inFeatures && trimmed.startsWith('-')) {
        features.push(trimmed.slice(1).trim())
      } else if (!inFeatures && !inUsage && trimmed && !trimmed.startsWith('-')) {
        descLines.push(trimmed)
      }
    }

    description = descLines.slice(0, 2).join(' ').trim()
    if (description.length > 200) {
      description = description.slice(0, 197) + '...'
    }
  }

  let category = 'Other'
  for (const [cat, examples] of Object.entries(CATEGORIES)) {
    if (examples.includes(name)) {
      category = cat
      break
    }
  }

  const screenshotPath = path.join(SCREENSHOTS_DIR, `${name}.png`)
  const hasScreenshot = fs.existsSync(screenshotPath)

  return {
    name,
    title,
    description,
    category,
    features,
    skip: name in SKIP_REASONS,
    skipReason: SKIP_REASONS[name],
    hasScreenshot,
    source: content,
  }
}

function getExamplesByCategory(): Map<string, ExampleMeta[]> {
  const result = new Map<string, ExampleMeta[]>()

  for (const cat of Object.keys(CATEGORIES)) {
    result.set(cat, [])
  }
  result.set('Other', [])

  let files: string[]
  try {
    files = fs.readdirSync(EXAMPLES_DIR)
      .filter(f => f.endsWith('.py') && !f.startsWith('__'))
  } catch {
    console.warn('[generate-examples] Could not read examples directory')
    return result
  }

  for (const file of files) {
    const meta = parseExampleMeta(path.join(EXAMPLES_DIR, file))
    if (meta) {
      const list = result.get(meta.category) || []
      list.push(meta)
      result.set(meta.category, list)
    }
  }

  for (const [cat, examples] of result) {
    const order = CATEGORIES[cat] || []
    examples.sort((a, b) => {
      const aIdx = order.indexOf(a.name)
      const bIdx = order.indexOf(b.name)
      if (aIdx === -1 && bIdx === -1) return a.name.localeCompare(b.name)
      if (aIdx === -1) return 1
      if (bIdx === -1) return -1
      return aIdx - bIdx
    })
  }

  for (const [cat, examples] of result) {
    if (examples.length === 0) {
      result.delete(cat)
    }
  }

  return result
}

function generateExampleMarkdown(meta: ExampleMeta, lang: 'en' | 'zh'): string {
  const lines: string[] = []

  lines.push(`### ${meta.title}`)
  lines.push('')

  if (meta.skip) {
    const label = lang === 'zh' ? '注意' : 'Note'
    lines.push(`::: warning ${label}`)
    lines.push(meta.skipReason || '')
    lines.push(':::')
    lines.push('')
  }

  if (meta.description) {
    lines.push(meta.description)
    lines.push('')
  }

  if (meta.hasScreenshot) {
    lines.push(`![${meta.title}](/examples/${meta.name}.png)`)
    lines.push('')
  }

  const detailsLabel = lang === 'zh' ? '查看源代码' : 'View Source Code'
  lines.push(`::: details ${detailsLabel}`)
  lines.push('```python')
  lines.push(meta.source.trim())
  lines.push('```')
  lines.push(':::')
  lines.push('')

  const runLabel = lang === 'zh' ? '运行' : 'Run'
  lines.push(`**${runLabel}:** \`python examples/${meta.name}.py\``)
  lines.push('')

  if (meta.features.length > 0) {
    const featuresLabel = lang === 'zh' ? '特性' : 'Features'
    lines.push(`**${featuresLabel}:**`)
    for (const feature of meta.features) {
      lines.push(`- ${feature}`)
    }
    lines.push('')
  }

  lines.push('---')
  lines.push('')

  return lines.join('\n')
}

export function generateExamplesDoc(lang: 'en' | 'zh' = 'en'): string {
  const examples = getExamplesByCategory()
  const lines: string[] = []

  lines.push('---')
  lines.push('outline: deep')
  lines.push('---')
  lines.push('')

  if (lang === 'zh') {
    lines.push('# 示例')
    lines.push('')
    lines.push('本页展示各种 AuroraView 示例，演示不同的功能和用例。')
    lines.push('')
    lines.push('::: tip 自动生成')
    lines.push('本页内容从 `examples/` 目录自动生成。')
    lines.push(':::')
  } else {
    lines.push('# Examples')
    lines.push('')
    lines.push('This page showcases various AuroraView examples demonstrating different features and use cases.')
    lines.push('')
    lines.push('::: tip Auto-generated')
    lines.push('This page is auto-generated from the `examples/` directory.')
    lines.push(':::')
  }
  lines.push('')

  for (const [category, exampleList] of examples) {
    lines.push(`## ${category}`)
    lines.push('')

    for (const meta of exampleList) {
      lines.push(generateExampleMarkdown(meta, lang))
    }
  }

  if (lang === 'zh') {
    lines.push('## 运行示例')
    lines.push('')
    lines.push('所有示例位于 `examples/` 目录：')
    lines.push('')
    lines.push('```bash')
    lines.push('# 运行任意示例')
    lines.push('python examples/<example_name>.py')
    lines.push('```')
    lines.push('')
    lines.push('## 生成截图')
    lines.push('')
    lines.push('使用以下命令为文档生成截图：')
    lines.push('')
    lines.push('```bash')
    lines.push('# 生成所有示例截图')
    lines.push('vx just example-screenshots')
    lines.push('')
    lines.push('# 生成特定示例截图')
    lines.push('vx just example-screenshot window_effects_demo')
    lines.push('')
    lines.push('# 列出可用示例')
    lines.push('vx just example-list')
    lines.push('```')
  } else {
    lines.push('## Running Examples')
    lines.push('')
    lines.push('All examples are located in the `examples/` directory:')
    lines.push('')
    lines.push('```bash')
    lines.push('# Run any example')
    lines.push('python examples/<example_name>.py')
    lines.push('```')
    lines.push('')
    lines.push('## Generate Screenshots')
    lines.push('')
    lines.push('Use the following commands to generate screenshots for documentation:')
    lines.push('')
    lines.push('```bash')
    lines.push('# Generate all example screenshots')
    lines.push('vx just example-screenshots')
    lines.push('')
    lines.push('# Generate specific example screenshot')
    lines.push('vx just example-screenshot window_effects_demo')
    lines.push('')
    lines.push('# List available examples')
    lines.push('vx just example-list')
    lines.push('```')
  }

  return lines.join('\n')
}

/**
 * Write generated examples docs to files.
 */
export function writeExamplesDocs(): void {
  console.log('[generate-examples] Generating examples documentation...')

  // English version
  const enContent = generateExamplesDoc('en')
  const enPath = path.join(DOCS_DIR, 'guide/examples.md')
  fs.writeFileSync(enPath, enContent, 'utf-8')
  console.log(`[generate-examples] Written: ${enPath}`)

  // Chinese version
  const zhContent = generateExamplesDoc('zh')
  const zhDir = path.join(DOCS_DIR, 'zh/guide')
  if (!fs.existsSync(zhDir)) {
    fs.mkdirSync(zhDir, { recursive: true })
  }
  const zhPath = path.join(zhDir, 'examples.md')
  fs.writeFileSync(zhPath, zhContent, 'utf-8')
  console.log(`[generate-examples] Written: ${zhPath}`)

  console.log('[generate-examples] Done!')
}

// Always run when this file is executed
writeExamplesDocs()
