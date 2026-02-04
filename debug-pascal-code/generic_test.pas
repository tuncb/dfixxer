unit GenericTest;

interface

uses
  System.Generics.Collections;

type
  TMy = class
  public
    procedure Foo;
  end;

implementation

procedure TMy.Foo;
var
  LObjects: TList<PObject>;
begin
  LObjects := TList<PObject>.Create();
end;

end.
