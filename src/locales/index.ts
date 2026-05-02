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

export function t(key: NestedKeyOf<Translations>): string {
  const keys = key.split('.')
  let result: any = locales[currentLocale]
  
  for (const k of keys) {
    if (result && typeof result === 'object' && k in result) {
      result = result[k]
    } else {
      console.warn(`Translation key not found: ${key}`)
      return key
    }
  }
  
  return typeof result === 'string' ? result : key
}

export function getTranslations(): Translations {
  return locales[currentLocale]
}
