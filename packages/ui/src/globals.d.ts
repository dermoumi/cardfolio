// Declare css module imports
declare module "*.css" {
  // classes are declared as any to work around noUncheckedIndexedAccess
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const classes: Record<string, any>;
  export default classes;
}
