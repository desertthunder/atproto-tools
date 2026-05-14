<script lang="ts">
  import { base } from '$app/paths';
  import { normalizeGraphHandle, socialGraphPath } from '$lib/graph/routes';
  import type { SocialGraphAvatarMode, SocialGraphNodeData } from '$lib/types/social-graph';

  type Props = {
    avatarMode?: SocialGraphAvatarMode;
    loading?: boolean;
    onClose?: () => void;
    onFetchSecondHop?: (profile: SocialGraphNodeData) => void;
    profile?: SocialGraphNodeData | null;
    secondHopHandles?: readonly string[];
  };

  let {
    avatarMode = 'rings',
    loading = false,
    onClose,
    onFetchSecondHop,
    profile = null,
    secondHopHandles = []
  }: Props = $props();

  const profileHref = $derived(profile ? `https://bsky.app/profile/${profile.handle.replace(/^@/, '')}` : '');
  const profileSecondHopHandle = $derived(profile ? normalizeGraphHandle(profile.handle).toLowerCase() : '');
  const ringAvatarUrl = $derived(profile ? `${base}/dicebear/rings/${profile.relationship}.svg` : '');
  const shiftOriginHref = $derived(profile ? socialGraphPath(profile.handle, 'all') : '');
  const shouldShowProfileAvatar = $derived(profile?.relationship === 'origin' || avatarMode === 'avatars');
  const avatarSrc = $derived(shouldShowProfileAvatar && profile?.avatarUrl ? profile.avatarUrl : ringAvatarUrl);
  const canFetchSecondHop = $derived(Boolean(profile && profile.relationship !== 'origin'));
  const hasSecondHop = $derived(secondHopHandles.includes(profileSecondHopHandle));

  const accentClasses = {
    follower: 'bg-blue-700',
    following: 'bg-rose-500',
    mutuals: 'bg-emerald-500',
    origin: 'bg-sky-500',
    'second-hop': 'bg-indigo-500'
  };

  const fallbackToRing = (event: Event) => {
    if (event.currentTarget instanceof HTMLImageElement) {
      event.currentTarget.src = ringAvatarUrl;
    }
  };

  const fetchSecondHop = () => {
    if (!profile || hasSecondHop) return;

    onFetchSecondHop?.(profile);
  };
</script>

{#if profile}
  <aside
    class="pointer-events-auto w-full overflow-hidden rounded-lg border border-blue-950 bg-black/90 backdrop-blur-xl">
    <div class="relative border-b border-blue-950 p-4">
      <button
        class="absolute top-3 right-3 grid h-6 w-6 place-items-center rounded border border-blue-950 bg-blue-950/30 text-xs text-blue-200/45 transition hover:bg-blue-950 hover:text-blue-50"
        onclick={onClose}
        aria-label="Close profile panel">×</button>
      <div
        class="mb-2.5 grid h-12 w-12 place-items-center overflow-hidden rounded-md border border-blue-900 bg-linear-to-br from-blue-300 via-blue-500 to-blue-900 text-xl font-bold text-white">
        <img class="h-full w-full object-cover" src={avatarSrc} alt="" onerror={fallbackToRing} />
      </div>
      <div class="mb-0.5 text-[15px] font-bold text-blue-50">{profile.displayName}</div>
      <a
        class="nodrag font-mono text-[11px] text-blue-300/70 transition hover:text-blue-200"
        href={profileHref}
        target="_blank"
        rel="external noopener noreferrer">
        @{profile.handle.replace(/^@/, '')}
      </a>
    </div>

    <div class="border-b border-blue-950 px-4 py-3 text-xs leading-6 text-blue-100/65">
      {profile.description ?? `${profile.displayName} is connected to this graph as ${profile.relationship}.`}
    </div>

    <div class="border-b border-blue-950 p-4">
      <div class="mb-2 text-[10px] tracking-widest text-blue-200/40 uppercase">Relationship</div>
      <div class="flex items-center gap-2 font-mono text-xs text-blue-100/70">
        <span class={`h-1.5 w-1.5 rounded-full ${accentClasses[profile.relationship]}`}></span>
        {profile.relationship}
      </div>
    </div>

    {#if canFetchSecondHop}
      <div class="grid gap-2 p-4">
        <button
          type="button"
          class="flex h-9 w-full items-center justify-center gap-1.5 rounded border border-blue-700 bg-blue-600/20 px-3 text-[12px] font-medium text-blue-50 transition hover:border-blue-400 hover:bg-blue-600/35 disabled:cursor-wait disabled:border-blue-950 disabled:bg-blue-950/35 disabled:text-blue-200/35"
          disabled={loading || hasSecondHop}
          onclick={fetchSecondHop}>
          <span class="flex items-center">
            <i class="i-tabler-git-branch"></i>
          </span>
          {hasSecondHop ? 'Second Hop Added' : 'Fetch Second Hop'}
        </button>
        <a
          class="flex h-9 w-full items-center justify-center gap-1.5 rounded border border-blue-900 bg-black px-3 text-[12px] font-medium text-blue-100 transition hover:border-blue-500 hover:bg-blue-950"
          href={shiftOriginHref}>
          <span class="flex items-center">
            <i class="i-tabler-route"></i>
          </span>
          Shift Origin
        </a>
      </div>
    {/if}
  </aside>
{/if}
