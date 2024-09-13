import en from './en.json';
import et from './et.json';
import fr from './fr.json';
import he from './he.json';
import it from './it.json';
import ko from './ko.json';
import ms_arab from './ms-arab.json';
import ms from './ms.json';
import pl from './pl.json';
import pt_br from './pt-br.json';
import qep from './qep.json';
import ro from './ro.json';
import ru from './ru.json';
import tr from './tr.json';
import vi from './vi.json';
import zh_cn from './zh-cn.json';
import zh_tw from './zh-tw.json';
import zh from './zh.json';

export const langs: Record<string, Record<string, string>> = {
	en,
	et,
	fr,
	he,
	it,
	ko,
	'ms-arab': ms_arab,
	ms,
	pl,
	'pt-br': pt_br,
	qep,
	ro,
	ru,
	tr,
	vi,
	'zh-cn': zh_cn,
	'zh-tw': zh_tw,
	zh,
};
