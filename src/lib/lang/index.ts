import en from "./en.json";
import et from "./et.json";
import it from "./it.json";
import ko from "./ko.json";
import ms from "./ms.json";
import vi from "./vi.json";
import zlm from "./zlm.json";
import zh_tw from "./zh-tw.json";

export const langs: Record<string, Record<string, string>> = {
    en,
    et,
    it,
    ko,
    ms,
    vi,
    zlm,
    "zh-tw": zh_tw
};
