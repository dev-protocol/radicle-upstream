<!--
 Copyright © 2021 The Radicle Upstream Contributors

 This file is part of radicle-upstream, distributed under the GPLv3
 with Radicle Linking Exception. For full terms see the included
 LICENSE file.
-->
<script lang="ts">
  import BranchIcon from "design-system/icons/Branch.svelte";

  export let name: string;
  export let style: string | undefined = undefined;
  export let dataCy: string | undefined = undefined;
</script>

<style>
  .branch {
    display: flex;
    gap: 0.25rem;
    align-items: center;
    padding: 0.5rem;
    background-color: var(--color-foreground-level-2);
    border-radius: 0.5rem;
  }
</style>

<div class="branch" {style} data-cy={dataCy}>
  <BranchIcon />
  <p class="typo-text-bold" style="color: var(--color-foreground-level-6);">
    {name}
  </p>
</div>
