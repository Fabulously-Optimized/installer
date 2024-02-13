import { langs } from './lang';

const locale = navigator.language.split('-')[0];
const defaultLocale = 'en';

export function trans(id: string, data?: Record<string, string | number | undefined>) {
	if (locale && langs[locale] && langs[locale][id]) {
		let text = langs[locale][id];
		for (const key in data) {
			if (Object.prototype.hasOwnProperty.call(data, key)) {
				const element = data[key];
				if (element !== undefined) text = text.replaceAll(`{{${key}}}`, element.toString());
			}
		}
		return text;
	}
	if (langs[defaultLocale] && langs[defaultLocale][id]) {
		let text = langs[defaultLocale][id];
		for (const key in data) {
			if (Object.prototype.hasOwnProperty.call(data, key)) {
				const element = data[key];
				if (element !== undefined) text = text.replaceAll(`{{${key}}}`, element.toString());
			}
		}
		return text;
	}
	return id;
}
