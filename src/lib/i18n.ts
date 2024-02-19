import { langs } from './lang';

function determineLocale(locale: string): string {
	if (langs[locale.toLowerCase()] != undefined) {
		return locale.toLowerCase()
	} else {
		return locale.toLowerCase().split('-')[0]
	}
}

const locale = determineLocale(navigator.language);
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
