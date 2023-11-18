<script setup lang="ts">
import { useSessionStore } from "../store";
import AccountList from "./AccountList.vue";

const store = useSessionStore();
</script>

<template>
    <template v-for="p in store.partitions" :key="p.slug">
        <AccountList :partition-slug="p.slug" />
    </template>
</template>