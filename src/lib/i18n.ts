import { locale as getLocale } from '@tauri-apps/api/os';
import { langs } from './lang';

let locale = navigator.language.split('-')[0];
const defaultLocale = "en";

getLocale().then(systemLocale => {
    if (systemLocale)
        locale = systemLocale.split('-')[0]
})

export function trans(id: string, data?: Record<string, string | number | undefined>) {
    if (locale && langs[locale] && langs[locale][id]) {
        const text = langs[locale][id];
        for (const key in data) {
            if (Object.prototype.hasOwnProperty.call(data, key)) {
                const element = data[key];
                if (element !== undefined)
                    text.replaceAll(`{{${key}}}`, element.toString())
            }
        }
        return text
    }
    if (langs[defaultLocale] && langs[defaultLocale][id]) {
        const text = langs[defaultLocale][id];
        for (const key in data) {
            if (Object.prototype.hasOwnProperty.call(data, key)) {
                const element = data[key];
                if (element !== undefined)
                    text.replaceAll(`{{${key}}}`, element.toString())
            }
        }
        return text
    }
    return id
}