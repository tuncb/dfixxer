unit UnaryPlusDemo;
interface
const
  A = +1;
  B = + 2;
implementation
begin
  A := +1;
  A := + Foo;
  A := +Foo(1);
  A := + (1 + 2);
  A := a + +b;
end.
