import { createApp } from "vue";
import "./styles.css";
import { createPinia } from "pinia";
import App from "./App.vue";

// Vuetify
import "@mdi/font/css/materialdesignicons.css";
import "vuetify/styles";
import { createVuetify } from "vuetify";
import { md3 } from "vuetify/blueprints";
import * as components from "vuetify/components";
import * as directives from "vuetify/directives";
import { aliases, mdi } from "vuetify/iconsets/mdi";
// 2023-11-19 no types are available for colors in vuetify 3
// @ts-expect-error-next-line
import colors from "vuetify/lib/util/colors";

const vuetify = createVuetify({
  components,
  blueprint: md3,
  theme: {
    themes: {
      light: {
        dark: false,
        colors: {
          primary: colors.teal.darken1,
          secondary: colors.teal.lighten4,
        },
      },
    },
  },
  icons: {
    defaultSet: "mdi",
    aliases,
    sets: { mdi },
  },
  directives,
});

createApp(App).use(createPinia()).use(vuetify).mount("#app");
