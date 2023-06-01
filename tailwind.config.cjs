const typography = require('@tailwindcss/typography');
const forms = require('@tailwindcss/forms');
const catppuccin = require('@catppuccin/tailwindcss');

/** @type {import('tailwindcss').Config}*/
const config = {
	content: ['./src/**/*.{html,js,svelte,ts}'],

	theme: {
		extend: {}
	},

	plugins: [forms, typography, catppuccin]
};

module.exports = config;
