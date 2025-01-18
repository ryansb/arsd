<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { nextTick } from "vue";
import { useSessionStore } from "../store";

const search = ref(false);
const searchTerms = ref("");
const searchBox = ref<HTMLInputElement | null>(null);
const store = useSessionStore();

const DEBOUNCE_MS = 250;
let timeout: number;

const handleKeydown = (event: KeyboardEvent) => {
  // on `Esc`, clear and hide the search box
  if (event.key === "Escape") {
    search.value = false;
    searchTerms.value = "";
    return;
  }

  if (
    event.target instanceof HTMLInputElement ||
    event.target instanceof HTMLTextAreaElement
  )
    return;

  // for regular characters, show and focus the search box
  // VTextField will capture the key press.
  if (event.key.length === 1) {
    search.value = true;
    nextTick(() => {
      searchBox.value?.focus();
    });
  }
};

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});

watch(searchTerms, (terms) => {
  timeout && clearTimeout(timeout);

  timeout = setTimeout(() => {
    if (terms && terms.length >= 1) store.$patch({ search_term: terms });
    else store.$patch({ search_term: undefined });
  }, DEBOUNCE_MS);
});

watch(search, (isShown) => {
  if (isShown) {
    nextTick(() => {
      searchBox.value && searchBox.value.focus();
    });
  }
});
</script>

<template>
    <VAppBarNavIcon @click="search = !search; searchTerms = ''">
        <VIcon>mdi-magnify</VIcon>
    </VAppBarNavIcon>
    <VTextField v-if="search" ref="searchBox" v-model="searchTerms" hide-details placeholder="Search" single-line
        clearable color="secondary" persistent-clear />
</template>