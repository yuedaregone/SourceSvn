import zhCN from './zh-CN'
import enUS from './en-US'

export type LocaleKey = keyof typeof zhCN
export type NestedKeyOf<T> = T extends object
  ? {
      [K in keyof T]: K extends string
        ? T[K] extends object
          ? `${K}.${NestedKeyOf<T[K]>}` | `${K}`
          : `${K}`
        : never
    }[keyof T]
  : never

export type Translations = typeof zhCN

const locales: Record<string, Translations> = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

let currentLocale = 'zh-CN'

export function setLocale(locale: string) {
  if (locales[locale]) {
    currentLocale = locale
  }
}

export function getLocale(): string {
  return currentLocale
}

export function t(key: NestedKeyOf<Translations>, params?: Record<string, string | number>): string {
  const keys = key.split('.')
  let result: unknown = locales[currentLocale]

  for (const k of keys) {
    if (result && typeof result === 'object' && k in result) {
      result = (result as Record<string, unknown>)[k]
    } else {
      console.warn(`Translation key not found: ${key}`)
      return key
    }
  }

  let text = typeof result === 'string' ? result : key
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(new RegExp(`\\{${k}\\}`, 'g'), String(v))
    }
  }
  return text
}

export function getTranslations(): Translations {
  return locales[currentLocale]
}
