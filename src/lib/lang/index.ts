import en from "./en.json";
import et from "./et.json";
import it from "./it.json";
import ko from "./ko.json";
import ms from "./ms.json";
import ms_arab from "./ms-arab.json";
import vi from "./vi.json";
import zh_tw from "./zh-tw.json";

export const langs: Record<string, Record<string, string>> = {
    en,
    et,
    it,
    ko,
    ms,
    "ms-arab": ms_arab,
    vi,
    "zh-tw": zh_tw
};
