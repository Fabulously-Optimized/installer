import { writable, derived } from 'svelte/store';
import { langs } from './lang';

const defaultLocale = 'en';

function determineLocale(locale: string): string {
	if (langs[locale.toLowerCase()] != undefined) {
		return locale.toLowerCase();
	} else if (langs[locale.toLowerCase().split('-')[0]] != undefined) {
		return locale.toLowerCase().split('-')[0];
	} else {
		return defaultLocale;
	}
}

export const locale = writable(determineLocale(navigator.language));

export const trans = derived(
	locale,
	($locale) => (id: string, data?: Record<string, string | number | undefined>) =>
		transInternal($locale, id, data)
);

export const dir = derived(locale, getDir);

export function getDir(locale: string) {
	return langs[locale][1] ? 'rtl' : 'ltr';
}

export const langIds: string[] = [];
for (const key in langs) {
	if (Object.prototype.hasOwnProperty.call(langs, key)) {
		langIds.push(key);
	}
}
export function langName(locale: string) {
	if (locale && langs[locale] && langs[locale][0]['name']) {
		return langs[locale][0]['name'];
	}
	return locale;
}

function transInternal(
	locale: string,
	id: string,
	data?: Record<string, string | number | undefined>
) {
	if (locale && langs[locale] && langs[locale][0][id]) {
		let text = langs[locale][0][id];
		for (const key in data) {
			if (Object.prototype.hasOwnProperty.call(data, key)) {
				const element = data[key];
				if (element !== undefined) text = text.replaceAll(`{{${key}}}`, element.toString());
			}
		}
		return text;
	}
	if (langs[defaultLocale] && langs[defaultLocale][0][id]) {
		let text = langs[defaultLocale][0][id];
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
