import be from './be.json';
import en from './en.json';
import es_es from './es-es.json';
import es_ve from './es-ve.json';
import et from './et.json';
import fr from './fr.json';
import he from './he.json';
import it from './it.json';
import ko from './ko.json';
import ms_arab from './ms-arab.json';
import ms from './ms.json';
import pl from './pl.json';
import pt_br from './pt-br.json';
import pt_pt from './pt-pt.json';
import qep from './qep.json';
import ro from './ro.json';
import ru from './ru.json';
import tr from './tr.json';
import uk from './uk.json';
import vi from './vi.json';
import zh_cn from './zh-cn.json';
import zh_hk from './zh-hk.json';
import zh_tw from './zh-tw.json';
import zh from './zh.json';

export const langs: Record<string, [Record<string, string>, boolean]> = {
	be: [be, false],
	en: [en, false],
	'es-es': [es_es, false],
	'es-ve': [es_ve, false],
	et: [et, false],
	fr: [fr, false],
	he: [he, true],
	it: [it, false],
	ko: [ko, false],
	ms: [ms, false],
	'ms-arab': [ms_arab, true],
	pl: [pl, false],
	'pt-br': [pt_br, false],
	'pt-pt': [pt_pt, false],
	qep: [qep, false],
	ro: [ro, false],
	ru: [ru, false],
	tr: [tr, false],
	uk: [uk, false],
	vi: [vi, false],
	'zh-cn': [zh_cn, false],
	'zh-hk': [zh_hk, false],
	'zh-tw': [zh_tw, false],
	zh: [zh, false]
};
