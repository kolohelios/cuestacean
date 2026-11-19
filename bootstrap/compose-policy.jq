# Build-only assembly of TWO independently materialized object modules (-s -S).
# Shared objects are namespaces; all other shared values are collisions, even
# equal leaves. This is neither CUE conjunction nor a residual/default merge.
def disjoint($left; $right):
  if ($left | type) != "object" or ($right | type) != "object" then
    error("module roots must be objects")
  else
    reduce ($right | keys_unsorted[]) as $key ($left;
      if has($key) then
        if (.[$key] | type) == "object" and ($right[$key] | type) == "object" then
          .[$key] = disjoint(.[$key]; $right[$key])
        else
          error("overlapping module leaf")
        end
      else
        .[$key] = $right[$key]
      end)
  end;
if type != "array" or length != 2 then
  error("exactly two object modules required")
else
  disjoint(.[0]; .[1])
end
