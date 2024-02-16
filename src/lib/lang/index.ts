import en from "./en.json";
import et from "./et.json";
import ko from "./ko.json";
import ms from "./ms.json";
import zlm from "./zlm.json";
import zh_tw from "./zh-TW.json";

export const langs: Record<string, Record<string, string>> = {
    en,
    et,
    ko,
    ms,
    zlm,
    "zh-TW": zh_tw
};
