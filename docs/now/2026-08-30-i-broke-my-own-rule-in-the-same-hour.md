# NOW -- I broke my own rule in the same hour (2026-08-30)

## I broke my own rule in the same hour (Refs #2931)

- skill 366 says the top first-error family in the C backlog was worth zero; that is wrong and §370 corrects it
- one emitted template, four lines, THREE cc errors: `call to undeclared function`, `variable has incomplete type 'void'`, `expected expression`
- errors two and three mention neither helper name, so a filter keyed on the name counted them as other families -- 0 of 166, when all three are the same construct
- grouping by message text is exactly what the census discipline forbids, and I wrote that rule into the audit prompt before breaking it in my own count
- corrected: cc accepts 174 as emitted, **265** with scaffold bodies emptied (+91 upper bound), **242** with the fix the sibling backends made (+68 honest) -- the largest single lever in the project
- the class was closed twice already, Zig in W585 and Verilog in W660, and W660's comment names its sibling and stops there; nobody grepped the C path
